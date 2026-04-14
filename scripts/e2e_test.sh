#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${PORT:-4545}"
HOST="${HOST:-127.0.0.1}"
TIMEOUT_SECS="${TIMEOUT_SECS:-12}"
WORK_DIR="${ROOT_DIR}/target/e2e"

SERVER_BIN="${ROOT_DIR}/myteams_server"
CLIENT_BIN="${ROOT_DIR}/myteams_cli"

SERVER_PID=""
CLIENT1_PID=""
CLIENT2_PID=""
FD1=""
FD2=""
LOG_SERVER=""
LOG_C1=""
LOG_C2=""
PIPE1=""
PIPE2=""
REPORT_FILE=""

TEST_TOTAL=0
TEST_PASSED=0
TEST_FAILED=0
TEST_CRASHED=0
declare -a TEST_STATUS=()
declare -a TEST_SECTION_CODE=()
declare -a TEST_SECTION_NAME=()
declare -a TEST_CODE=()
declare -a TEST_NAME=()
declare -a TEST_DETAILS=()
declare -a SECTION_ORDER=()

USER1_UUID=""
USER2_UUID=""
TEAM_UUID=""
CHANNEL_UUID=""
THREAD_UUID=""
TEAM_NAME=""
CHAN_NAME=""
THREAD_TITLE=""

fail() {
  echo "[FAIL] $*" >&2
  exit 1
}

log() {
  echo "[INFO] $*"
}

ensure_section_registered() {
  local section_code="$1"
  local section_name="$2"
  local key="${section_code}|${section_name}"
  local existing

  for existing in "${SECTION_ORDER[@]}"; do
    if [[ "${existing}" == "${key}" ]]; then
      return 0
    fi
  done

  SECTION_ORDER+=("${key}")
}

record_test_result() {
  local status="$1"
  local section_code="$2"
  local section_name="$3"
  local code="$4"
  local name="$5"
  local details="$6"

  TEST_TOTAL=$((TEST_TOTAL + 1))
  if [[ "${status}" == "PASS" ]]; then
    TEST_PASSED=$((TEST_PASSED + 1))
  elif [[ "${status}" == "CRASH" ]]; then
    TEST_CRASHED=$((TEST_CRASHED + 1))
    TEST_FAILED=$((TEST_FAILED + 1))
  else
    TEST_FAILED=$((TEST_FAILED + 1))
  fi

  TEST_STATUS+=("${status}")
  TEST_SECTION_CODE+=("${section_code}")
  TEST_SECTION_NAME+=("${section_name}")
  TEST_CODE+=("${code}")
  TEST_NAME+=("${name}")
  TEST_DETAILS+=("${details}")
}

run_named_test() {
  local section_code="$1"
  local section_name="$2"
  local code="$3"
  local name="$4"
  local fn="$5"
  local rc

  ensure_section_registered "${section_code}" "${section_name}"
  log "Running test: ${code} :: ${name}"

  if "${fn}"; then
    record_test_result "PASS" "${section_code}" "${section_name}" "${code}" "${name}" "PASSED"
  else
    rc="$?"
    if (( rc >= 128 )); then
      record_test_result "CRASH" "${section_code}" "${section_name}" "${code}" "${name}" "Invalid exit status ${rc}"
    else
      record_test_result "FAIL" "${section_code}" "${section_name}" "${code}" "${name}" "FAILED"
    fi
  fi
}

print_detailed_report() {
  echo "=== E2E Expanded Report ==="
  echo "Generated at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  echo

  local section_key section_code section_name i
  local sec_total sec_passed sec_failed sec_crashed sec_percent

  for section_key in "${SECTION_ORDER[@]}"; do
    section_code="${section_key%%|*}"
    section_name="${section_key#*|}"

    sec_total=0
    sec_passed=0
    sec_failed=0
    sec_crashed=0

    for ((i = 0; i < ${#TEST_STATUS[@]}; i++)); do
      if [[ "${TEST_SECTION_CODE[$i]}" == "${section_code}" && "${TEST_SECTION_NAME[$i]}" == "${section_name}" ]]; then
        sec_total=$((sec_total + 1))
        if [[ "${TEST_STATUS[$i]}" == "PASS" ]]; then
          sec_passed=$((sec_passed + 1))
        elif [[ "${TEST_STATUS[$i]}" == "CRASH" ]]; then
          sec_crashed=$((sec_crashed + 1))
          sec_failed=$((sec_failed + 1))
        else
          sec_failed=$((sec_failed + 1))
        fi
      fi
    done

    if (( sec_total > 0 )); then
      sec_percent=$((sec_passed * 100 / sec_total))
    else
      sec_percent=0
    fi

    echo "${section_code} - ${section_name}"
    echo "${sec_percent}% Passed"
    echo "Total: ${sec_total}"
    echo "Passed: ${sec_passed}"
    echo "Crashed: ${sec_crashed}"
    echo "Failed or skipped: ${sec_failed}"

    for ((i = 0; i < ${#TEST_STATUS[@]}; i++)); do
      if [[ "${TEST_SECTION_CODE[$i]}" == "${section_code}" && "${TEST_SECTION_NAME[$i]}" == "${section_name}" ]]; then
        echo "${TEST_CODE[$i]} - ${TEST_NAME[$i]} -> ${TEST_DETAILS[$i]}"
      fi
    done
    echo
  done

  echo
  echo "Global summary: total=${TEST_TOTAL}, passed=${TEST_PASSED}, crashed=${TEST_CRASHED}, failed_or_skipped=${TEST_FAILED}"
}

cleanup() {
  set +e

  if [[ -n "${FD1}" ]]; then
    eval "exec ${FD1}>&-"
  fi
  if [[ -n "${FD2}" ]]; then
    eval "exec ${FD2}>&-"
  fi

  [[ -n "${CLIENT1_PID}" ]] && kill "${CLIENT1_PID}" >/dev/null 2>&1 || true
  [[ -n "${CLIENT2_PID}" ]] && kill "${CLIENT2_PID}" >/dev/null 2>&1 || true
  [[ -n "${SERVER_PID}" ]] && kill "${SERVER_PID}" >/dev/null 2>&1 || true

  [[ -n "${CLIENT1_PID}" ]] && wait "${CLIENT1_PID}" >/dev/null 2>&1 || true
  [[ -n "${CLIENT2_PID}" ]] && wait "${CLIENT2_PID}" >/dev/null 2>&1 || true
  [[ -n "${SERVER_PID}" ]] && wait "${SERVER_PID}" >/dev/null 2>&1 || true

  [[ -n "${PIPE1}" ]] && rm -f "${PIPE1}"
  [[ -n "${PIPE2}" ]] && rm -f "${PIPE2}"
}

trap cleanup EXIT

line_count() {
  local file="$1"
  if [[ ! -f "${file}" ]]; then
    echo 0
    return
  fi
  wc -l < "${file}" | tr -d ' '
}

wait_for_new_pattern() {
  local file="$1"
  local start_line="$2"
  local pattern="$3"
  local timeout="$4"

  local start_ts now
  start_ts="$(date +%s)"

  while true; do
    if tail -n +"$((start_line + 1))" "${file}" | grep -E -q "${pattern}"; then
      return 0
    fi

    now="$(date +%s)"
    if (( now - start_ts >= timeout )); then
      return 1
    fi
    sleep 0.1
  done
}

send_and_expect() {
  local fd="$1"
  local file="$2"
  local cmd="$3"
  local pattern="$4"

  local before
  before="$(line_count "${file}")"

  printf '%s\n' "${cmd}" >&"${fd}"

  if ! wait_for_new_pattern "${file}" "${before}" "${pattern}" "${TIMEOUT_SECS}"; then
    echo "[DEBUG] Last lines from ${file}:" >&2
    tail -n 80 "${file}" >&2 || true
    echo "[ERROR] Timed out waiting for pattern '${pattern}' after command: ${cmd}" >&2
    return 1
  fi

  return 0
}

expect_async_in_log() {
  local file="$1"
  local start_line="$2"
  local pattern="$3"

  if ! wait_for_new_pattern "${file}" "${start_line}" "${pattern}" "${TIMEOUT_SECS}"; then
    echo "[DEBUG] Last lines from ${file}:" >&2
    tail -n 80 "${file}" >&2 || true
    echo "[ERROR] Timed out waiting for async pattern '${pattern}'" >&2
    return 1
  fi

  return 0
}

extract_first_match_after() {
  local file="$1"
  local start_line="$2"
  local pattern="$3"

  tail -n +"$((start_line + 1))" "${file}" | grep -E -m1 -o "${pattern}" || true
}

send_expect_and_capture() {
  local fd="$1"
  local file="$2"
  local cmd="$3"
  local wait_pattern="$4"
  local capture_pattern="$5"

  local before captured
  before="$(line_count "${file}")"

  printf '%s\n' "${cmd}" >&"${fd}"

  if ! wait_for_new_pattern "${file}" "${before}" "${wait_pattern}" "${TIMEOUT_SECS}"; then
    echo "[DEBUG] Last lines from ${file}:" >&2
    tail -n 80 "${file}" >&2 || true
    echo "[ERROR] Timed out waiting for pattern '${wait_pattern}' after command: ${cmd}" >&2
    return 1
  fi

  captured="$(extract_first_match_after "${file}" "${before}" "${capture_pattern}")"
  if [[ -z "${captured}" ]]; then
    echo "[DEBUG] Last lines from ${file}:" >&2
    tail -n 80 "${file}" >&2 || true
    echo "[ERROR] Could not capture pattern '${capture_pattern}' after command: ${cmd}" >&2
    return 1
  fi

  echo "${captured}"
}

require_binaries() {
  if [[ -x "${SERVER_BIN}" && -x "${CLIENT_BIN}" ]]; then
    return
  fi

  log "Binaries not found at project root, building in release mode"
  (cd "${ROOT_DIR}" && make all)

  [[ -x "${SERVER_BIN}" ]] || fail "Missing server binary: ${SERVER_BIN}"
  [[ -x "${CLIENT_BIN}" ]] || fail "Missing client binary: ${CLIENT_BIN}"
}

prepare_runtime() {
  mkdir -p "${WORK_DIR}"
  mkdir -p "${ROOT_DIR}/data"
  LOG_SERVER="${WORK_DIR}/server.log"
  LOG_C1="${WORK_DIR}/client1.log"
  LOG_C2="${WORK_DIR}/client2.log"
  PIPE1="${WORK_DIR}/client1.in"
  PIPE2="${WORK_DIR}/client2.in"
  REPORT_FILE="${WORK_DIR}/final_report.txt"

  : > "${LOG_SERVER}"
  : > "${LOG_C1}"
  : > "${LOG_C2}"

  rm -f "${PIPE1}" "${PIPE2}"
  mkfifo "${PIPE1}" "${PIPE2}"

  rm -rf "${ROOT_DIR}/data/"
}

start_processes() {
  local lib_dir
  lib_dir="${ROOT_DIR}/libs/myteams"
  export LD_LIBRARY_PATH="${lib_dir}:${LD_LIBRARY_PATH:-}"

  log "Starting server on ${HOST}:${PORT}"
  "${SERVER_BIN}" "${PORT}" >"${LOG_SERVER}" 2>&1 &
  SERVER_PID="$!"

  local start_ts
  start_ts="$(date +%s)"
  while true; do
    if grep -q "Server listening" "${LOG_SERVER}" 2>/dev/null; then
      break
    fi
    if ! kill -0 "${SERVER_PID}" 2>/dev/null; then
      tail -n 80 "${LOG_SERVER}" >&2 || true
      fail "Server process exited unexpectedly"
    fi
    if (( $(date +%s) - start_ts >= TIMEOUT_SECS )); then
      tail -n 80 "${LOG_SERVER}" >&2 || true
      fail "Timed out waiting for server startup"
    fi
    sleep 0.1
  done

  exec {FD1}<>"${PIPE1}"
  exec {FD2}<>"${PIPE2}"

  log "Starting client #1"
  "${CLIENT_BIN}" "${HOST}" "${PORT}" <"${PIPE1}" >"${LOG_C1}" 2>&1 &
  CLIENT1_PID="$!"

  log "Starting client #2"
  "${CLIENT_BIN}" "${HOST}" "${PORT}" <"${PIPE2}" >"${LOG_C2}" 2>&1 &
  CLIENT2_PID="$!"

  wait_for_new_pattern "${LOG_C1}" 0 "myteams >" "${TIMEOUT_SECS}" || fail "Client #1 prompt not detected"
  wait_for_new_pattern "${LOG_C2}" 0 "myteams >" "${TIMEOUT_SECS}" || fail "Client #2 prompt not detected"
}

test_authentication_and_users() {
  USER1_UUID="$(send_expect_and_capture "${FD1}" "${LOG_C1}" '/login "alice"' 'R200' '[0-9a-fA-F-]{36}')" || return 1
  USER2_UUID="$(send_expect_and_capture "${FD2}" "${LOG_C2}" '/login "bob"' 'R200' '[0-9a-fA-F-]{36}')" || return 1

  [[ -n "${USER1_UUID}" ]] || return 1
  [[ -n "${USER2_UUID}" ]] || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/users' 'R200 .*USERS.*alice.*bob' || return 1
  return 0
}

test_preliminaries_process_health() {
  kill -0 "${SERVER_PID}" 2>/dev/null || return 1
  kill -0 "${CLIENT1_PID}" 2>/dev/null || return 1
  kill -0 "${CLIENT2_PID}" 2>/dev/null || return 1
  return 0
}

test_preliminaries_logs_ready() {
  grep -q 'Server listening' "${LOG_SERVER}" || return 1
  grep -q 'myteams >' "${LOG_C1}" || return 1
  grep -q 'myteams >' "${LOG_C2}" || return 1
  return 0
}

test_preliminaries_unknown_command() {
  send_and_expect "${FD1}" "${LOG_C1}" '/definitely_not_a_command' 'unknown command' || return 1
  return 0
}

test_auth_lookup_existing_user() {
  send_and_expect "${FD1}" "${LOG_C1}" "/user \"${USER2_UUID}\"" "R200 .*USER.*${USER2_UUID}.*bob" || return 1
  return 0
}

test_auth_lookup_unknown_user() {
  local unknown_user_uuid
  unknown_user_uuid="00000000-0000-0000-0000-0000000000ab"
  send_and_expect "${FD1}" "${LOG_C1}" "/user \"${unknown_user_uuid}\"" "R404 .*${unknown_user_uuid}" || return 1
  return 0
}

test_private_messages() {
  local c2_before_msg
  c2_before_msg="$(line_count "${LOG_C2}")"
  send_and_expect "${FD1}" "${LOG_C1}" "/send \"${USER2_UUID}\" \"hello from alice\"" 'R200' || return 1
  expect_async_in_log "${LOG_C2}" "${c2_before_msg}" 'I100 NEW_MESSAGE .*hello from alice' || return 1
  send_and_expect "${FD2}" "${LOG_C2}" "/messages \"${USER1_UUID}\"" 'R200 .*MESSAGES.*hello from alice' || return 1
  return 0
}

test_message_non_existing_user() {
  local unknown_user_uuid
  unknown_user_uuid="00000000-0000-0000-0000-0000000000aa"

  send_and_expect "${FD1}" "${LOG_C1}" "/send \"${unknown_user_uuid}\" \"should fail\"" "R404 .*${unknown_user_uuid}" || return 1
  send_and_expect "${FD1}" "${LOG_C1}" "/messages \"${unknown_user_uuid}\"" "R404 .*${unknown_user_uuid}" || return 1
  return 0
}

test_messaging_invalid_arguments() {
  send_and_expect "${FD1}" "${LOG_C1}" '/send' 'command error: invalid argument count for /send' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/messages' 'command error: invalid argument count for /messages' || return 1
  return 0
}

test_messaging_malformed_user_identifiers() {
  send_and_expect "${FD1}" "${LOG_C1}" '/send "not-a-uuid" "bad id"' 'R404 .*not-a-uuid' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/messages "not-a-uuid"' 'R404 .*not-a-uuid' || return 1
  return 0
}

test_team_and_subscription_flow() {
  TEAM_UUID="$(send_expect_and_capture "${FD1}" "${LOG_C1}" "/create \"${TEAM_NAME}\" \"team description\"" 'R200 .*TEAM' '[0-9a-fA-F-]{36}')" || return 1
  [[ -n "${TEAM_UUID}" ]] || return 1

  send_and_expect "${FD1}" "${LOG_C1}" "/subscribe \"${TEAM_UUID}\"" "R200 .*SUBSCRIBED.*${TEAM_UUID}" || return 1
  send_and_expect "${FD2}" "${LOG_C2}" "/subscribe \"${TEAM_UUID}\"" "R200 .*SUBSCRIBED.*${TEAM_UUID}" || return 1
  send_and_expect "${FD1}" "${LOG_C1}" "/subscribed \"${TEAM_UUID}\"" 'R200 .*USERS' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/subscribed' "R200 .*TEAMS.*${TEAM_UUID}" || return 1
  return 0
}

test_channel_creation_broadcast() {
  local c2_before_chan
  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\"" 'R200' || return 1
  c2_before_chan="$(line_count "${LOG_C2}")"
  CHANNEL_UUID="$(send_expect_and_capture "${FD1}" "${LOG_C1}" "/create \"${CHAN_NAME}\" \"channel description\"" 'R200 .*CHANNEL' '[0-9a-fA-F-]{36}')" || return 1
  [[ -n "${CHANNEL_UUID}" ]] || return 1
  expect_async_in_log "${LOG_C2}" "${c2_before_chan}" "I100 NEW_CHANNEL .*${CHANNEL_UUID}.*${CHAN_NAME}" || return 1
  return 0
}

test_thread_and_reply_flow() {
  local c2_before_thread c1_before_reply

  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\" \"${CHANNEL_UUID}\"" 'R200' || return 1
  c2_before_thread="$(line_count "${LOG_C2}")"
  THREAD_UUID="$(send_expect_and_capture "${FD1}" "${LOG_C1}" "/create \"${THREAD_TITLE}\" \"thread body\"" 'R200 .*THREAD' '[0-9a-fA-F-]{36}')" || return 1
  [[ -n "${THREAD_UUID}" ]] || return 1
  expect_async_in_log "${LOG_C2}" "${c2_before_thread}" "I100 NEW_THREAD .*${THREAD_UUID}.*${THREAD_TITLE}" || return 1

  send_and_expect "${FD2}" "${LOG_C2}" "/use \"${TEAM_UUID}\" \"${CHANNEL_UUID}\" \"${THREAD_UUID}\"" 'R200' || return 1
  c1_before_reply="$(line_count "${LOG_C1}")"
  send_and_expect "${FD2}" "${LOG_C2}" '/create "reply from bob"' 'R200 .*REPLY' || return 1
  expect_async_in_log "${LOG_C1}" "${c1_before_reply}" "I100 NEW_REPLY .*${TEAM_UUID}.*${THREAD_UUID}.*reply from bob" || return 1
  return 0
}

test_context_list_at_root() {
  send_and_expect "${FD1}" "${LOG_C1}" '/use' 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/list' "R200 .*TEAMS.*${TEAM_UUID}.*${TEAM_NAME}" || return 1
  return 0
}

test_context_info_at_root() {
  send_and_expect "${FD1}" "${LOG_C1}" '/use' 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/info' "R200 .*USER.*${USER1_UUID}.*alice" || return 1
  return 0
}

test_context_list_at_team() {
  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\"" 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/list' "R200 .*CHANNELS.*${CHANNEL_UUID}.*${CHAN_NAME}" || return 1
  return 0
}

test_context_info_at_team() {
  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\"" 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/info' "R200 .*TEAM.*${TEAM_UUID}.*${TEAM_NAME}" || return 1
  return 0
}

test_context_list_at_channel() {
  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\" \"${CHANNEL_UUID}\"" 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/list' "R200 .*THREADS.*${THREAD_UUID}.*${THREAD_TITLE}" || return 1
  return 0
}

test_context_info_at_channel() {
  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\" \"${CHANNEL_UUID}\"" 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/info' "R200 .*CHANNEL.*${CHANNEL_UUID}.*${CHAN_NAME}" || return 1
  return 0
}

test_context_list_at_thread() {
  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\" \"${CHANNEL_UUID}\" \"${THREAD_UUID}\"" 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/list' 'R200 .*REPLIES.*reply from bob' || return 1
  return 0
}

test_context_info_at_thread() {
  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\" \"${CHANNEL_UUID}\" \"${THREAD_UUID}\"" 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/info' "R200 .*THREAD.*${THREAD_UUID}.*${THREAD_TITLE}" || return 1
  return 0
}

test_context_invalid_use_targets() {
  local unknown_channel_uuid unknown_thread_uuid
  unknown_channel_uuid="00000000-0000-0000-0000-0000000000bb"
  unknown_thread_uuid="00000000-0000-0000-0000-0000000000cc"

  send_and_expect "${FD1}" "${LOG_C1}" '/use "00000000-0000-0000-0000-000000000000"' 'R404 .*00000000-0000-0000-0000-000000000000' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\" \"${unknown_channel_uuid}\"" "R404 .*${unknown_channel_uuid}" || return 1
  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\" \"${CHANNEL_UUID}\" \"${unknown_thread_uuid}\"" "R404 .*${unknown_thread_uuid}" || return 1
  return 0
}

test_resource_invalid_create_arity() {
  send_and_expect "${FD1}" "${LOG_C1}" '/use' 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/create "only-team-name"' 'command error: invalid argument count for /create' || return 1

  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\"" 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/create "only-channel-name"' 'command error: invalid argument count for /create' || return 1

  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\" \"${CHANNEL_UUID}\"" 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/create "only-thread-title"' 'command error: invalid argument count for /create' || return 1

  send_and_expect "${FD1}" "${LOG_C1}" "/use \"${TEAM_UUID}\" \"${CHANNEL_UUID}\" \"${THREAD_UUID}\"" 'R200' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/create "reply body" "too much"' 'command error: invalid argument count for /create' || return 1
  return 0
}

test_resource_unknown_subscription_targets() {
  send_and_expect "${FD1}" "${LOG_C1}" '/subscribe "not-a-uuid"' 'R404 .*not-a-uuid' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/subscribed "not-a-uuid"' 'R404 .*not-a-uuid' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/unsubscribe "not-a-uuid"' 'R404 .*not-a-uuid' || return 1
  return 0
}

test_context_subscription_argument_errors() {
  send_and_expect "${FD1}" "${LOG_C1}" '/subscribe' 'command error: invalid argument count for /subscribe' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/unsubscribe' 'command error: invalid argument count for /unsubscribe' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/subscribed "a" "b"' 'command error: invalid argument count for /subscribed' || return 1
  return 0
}

test_context_invalid_use_arity() {
  send_and_expect "${FD1}" "${LOG_C1}" '/use "a" "b" "c" "d"' 'command error: invalid argument count for /use' || return 1
  return 0
}

test_context_invalid_list_info_arity() {
  send_and_expect "${FD1}" "${LOG_C1}" '/list "extra"' 'command error: invalid argument count for /list' || return 1
  send_and_expect "${FD1}" "${LOG_C1}" '/info "extra"' 'command error: invalid argument count for /info' || return 1
  return 0
}

test_unsubscribe_cleanup() {
  send_and_expect "${FD2}" "${LOG_C2}" "/unsubscribe \"${TEAM_UUID}\"" "R200 .*UNSUBSCRIBED.*${TEAM_UUID}" || return 1
  return 0
}

run_scenario() {
  local suffix
  suffix="$(date +%s)"
  TEAM_NAME="qa_team_${suffix}"
  CHAN_NAME="qa_chan_${suffix}"
  THREAD_TITLE="qa_thread_${suffix}"

  run_named_test "01" "Preliminaries" "0101" "processes are alive" test_preliminaries_process_health
  run_named_test "01" "Preliminaries" "0102" "logs and prompts are ready" test_preliminaries_logs_ready
  run_named_test "01" "Preliminaries" "0103" "unknown command" test_preliminaries_unknown_command

  run_named_test "02" "Authentication" "0201" "basic login and users listing" test_authentication_and_users
  run_named_test "02" "Authentication" "0202" "lookup existing user" test_auth_lookup_existing_user
  run_named_test "02" "Authentication" "0203" "lookup non-existing user" test_auth_lookup_unknown_user

  run_named_test "03" "Messaging" "0301" "direct message delivery" test_private_messages
  run_named_test "03" "Messaging" "0302" "message non-existing user" test_message_non_existing_user
  run_named_test "03" "Messaging" "0303" "invalid messaging arguments" test_messaging_invalid_arguments
  run_named_test "03" "Messaging" "0304" "malformed user identifiers" test_messaging_malformed_user_identifiers

  run_named_test "04" "Resources" "0401" "team subscription flow" test_team_and_subscription_flow
  run_named_test "04" "Resources" "0402" "channel creation broadcast" test_channel_creation_broadcast
  run_named_test "04" "Resources" "0403" "thread and reply flow" test_thread_and_reply_flow
  run_named_test "04" "Resources" "0404" "invalid create argument counts" test_resource_invalid_create_arity
  run_named_test "04" "Resources" "0405" "unknown subscription targets" test_resource_unknown_subscription_targets

  run_named_test "05" "Context" "0501" "list at root context" test_context_list_at_root
  run_named_test "05" "Context" "0502" "info at root context" test_context_info_at_root
  run_named_test "05" "Context" "0503" "list at team context" test_context_list_at_team
  run_named_test "05" "Context" "0504" "info at team context" test_context_info_at_team
  run_named_test "05" "Context" "0505" "list at channel context" test_context_list_at_channel
  run_named_test "05" "Context" "0506" "info at channel context" test_context_info_at_channel
  run_named_test "05" "Context" "0507" "list at thread context" test_context_list_at_thread
  run_named_test "05" "Context" "0508" "info at thread context" test_context_info_at_thread
  run_named_test "05" "Context" "0509" "invalid use targets" test_context_invalid_use_targets
  run_named_test "05" "Context" "0510" "invalid subscription arguments" test_context_subscription_argument_errors
  run_named_test "05" "Context" "0511" "invalid use arity" test_context_invalid_use_arity
  run_named_test "05" "Context" "0512" "invalid list/info arity" test_context_invalid_list_info_arity

  run_named_test "07" "Cleanup" "0701" "unsubscribe" test_unsubscribe_cleanup
}

main() {
  require_binaries
  prepare_runtime
  start_processes
  run_scenario
  print_detailed_report | tee "${REPORT_FILE}"
  echo "[INFO] Final report written to ${REPORT_FILE}"

  if (( TEST_FAILED > 0 )); then
    fail "End-to-end run completed with failing tests"
  fi

  echo "[PASS] Full end-to-end scenario completed"
}

main "$@"
