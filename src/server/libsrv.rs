#![allow(dead_code)]

use std::os::raw::{c_char, c_int};

unsafe extern "C" {
    pub fn server_event_team_created(
        team_uuid: *const c_char,
        team_name: *const c_char,
        user_uuid: *const c_char,
    ) -> c_int;

    pub fn server_event_channel_created(
        team_uuid: *const c_char,
        channel_uuid: *const c_char,
        channel_name: *const c_char,
    ) -> c_int;

    pub fn server_event_thread_created(
        channel_uuid: *const c_char,
        thread_uuid: *const c_char,
        user_uuid: *const c_char,
        thread_title: *const c_char,
        thread_body: *const c_char,
    ) -> c_int;

    pub fn server_event_reply_created(
        thread_uuid: *const c_char,
        user_uuid: *const c_char,
        reply_body: *const c_char,
    ) -> c_int;

    pub fn server_event_user_subscribed(
        team_uuid: *const c_char,
        user_uuid: *const c_char,
    ) -> c_int;

    pub fn server_event_user_unsubscribed(
        team_uuid: *const c_char,
        user_uuid: *const c_char,
    ) -> c_int;

    pub fn server_event_user_created(user_uuid: *const c_char, user_name: *const c_char) -> c_int;

    pub fn server_event_user_loaded(user_uuid: *const c_char, user_name: *const c_char) -> c_int;

    pub fn server_event_user_logged_in(user_uuid: *const c_char) -> c_int;

    pub fn server_event_user_logged_out(user_uuid: *const c_char) -> c_int;

    pub fn server_event_private_message_sended(
        sender_uuid: *const c_char,
        receiver_uuid: *const c_char,
        message_body: *const c_char,
    ) -> c_int;
}
