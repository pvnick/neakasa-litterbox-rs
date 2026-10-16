#![allow(non_snake_case)]
mod client_cases;
mod pure;
mod stream_cases;
mod structural;
mod support;
mod timing;
mod transport_cases;
mod aliyun_handshake {
    #[tokio::test]
    async fn test_full_handshake_returns_iot_session_with_endpoint() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_full_handshake_returns_iot_session_with_endpoint",
            super::client_cases::run(
                "aliyun_handshake",
                "test_full_handshake_returns_iot_session_with_endpoint",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_empty_auth_token_raises_before_network() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_empty_auth_token_raises_before_network",
            super::client_cases::run(
                "aliyun_handshake",
                "test_empty_auth_token_raises_before_network",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_region_non_200_raises_apierror() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_region_non_200_raises_apierror",
            super::client_cases::run("aliyun_handshake", "test_region_non_200_raises_apierror"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_region_missing_endpoints_raises() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_region_missing_endpoints_raises",
            super::client_cases::run("aliyun_handshake", "test_region_missing_endpoints_raises"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_vid_top_level_failure_raises() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_vid_top_level_failure_raises",
            super::client_cases::run("aliyun_handshake", "test_vid_top_level_failure_raises"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_vid_inner_failure_raises() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_vid_inner_failure_raises",
            super::client_cases::run("aliyun_handshake", "test_vid_inner_failure_raises"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_vid_missing_value_raises() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_vid_missing_value_raises",
            super::client_cases::run("aliyun_handshake", "test_vid_missing_value_raises"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_sid_failure_raises_authentication_error() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_sid_failure_raises_authentication_error",
            super::client_cases::run(
                "aliyun_handshake",
                "test_sid_failure_raises_authentication_error",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_sid_missing_value_raises_authentication_error() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_sid_missing_value_raises_authentication_error",
            super::client_cases::run(
                "aliyun_handshake",
                "test_sid_missing_value_raises_authentication_error",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_create_token_non_200_raises_authentication_error() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_create_token_non_200_raises_authentication_error",
            super::client_cases::run(
                "aliyun_handshake",
                "test_create_token_non_200_raises_authentication_error",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_create_token_missing_value_raises() {
        super::timing::asynchronous(
            "aligned::aliyun_handshake::test_create_token_missing_value_raises",
            super::client_cases::run("aliyun_handshake", "test_create_token_missing_value_raises"),
        )
        .await;
    }
    #[test]
    fn test_get_code_rejects_bool_and_non_int() {
        super::timing::sync(
            "aligned::aliyun_handshake::test_get_code_rejects_bool_and_non_int",
            || super::pure::run("aliyun_handshake", "test_get_code_rejects_bool_and_non_int"),
        );
    }
}
mod aliyun_session_refresh {
    #[test]
    fn test_unwrap_aliyun_maps_401_to_session_expired() {
        super::timing::sync(
            "aligned::aliyun_session_refresh::test_unwrap_aliyun_maps_401_to_session_expired",
            || {
                super::pure::run(
                    "aliyun_session_refresh",
                    "test_unwrap_aliyun_maps_401_to_session_expired",
                )
            },
        );
    }
    #[test]
    fn test_unwrap_aliyun_other_errors_stay_apierror() {
        super::timing::sync(
            "aligned::aliyun_session_refresh::test_unwrap_aliyun_other_errors_stay_apierror",
            || {
                super::pure::run(
                    "aliyun_session_refresh",
                    "test_unwrap_aliyun_other_errors_stay_apierror",
                )
            },
        );
    }
    #[test]
    fn test_unwrap_aliyun_returns_data_on_success() {
        super::timing::sync(
            "aligned::aliyun_session_refresh::test_unwrap_aliyun_returns_data_on_success",
            || {
                super::pure::run(
                    "aliyun_session_refresh",
                    "test_unwrap_aliyun_returns_data_on_success",
                )
            },
        );
    }
    #[tokio::test]
    async fn test_aliyun_call_authed_refreshes_on_401() {
        super::timing::asynchronous(
            "aligned::aliyun_session_refresh::test_aliyun_call_authed_refreshes_on_401",
            super::client_cases::run(
                "aliyun_session_refresh",
                "test_aliyun_call_authed_refreshes_on_401",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_aliyun_call_authed_propagates_second_401() {
        super::timing::asynchronous(
            "aligned::aliyun_session_refresh::test_aliyun_call_authed_propagates_second_401",
            super::client_cases::run(
                "aliyun_session_refresh",
                "test_aliyun_call_authed_propagates_second_401",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_aliyun_call_authed_passes_through_on_first_success() {
        super::timing::asynchronous("aligned::aliyun_session_refresh::test_aliyun_call_authed_passes_through_on_first_success", super::client_cases::run("aliyun_session_refresh","test_aliyun_call_authed_passes_through_on_first_success")).await;
    }
    #[tokio::test]
    async fn test_aliyun_call_authed_uses_current_regional_host() {
        super::timing::asynchronous(
            "aligned::aliyun_session_refresh::test_aliyun_call_authed_uses_current_regional_host",
            super::client_cases::run(
                "aliyun_session_refresh",
                "test_aliyun_call_authed_uses_current_regional_host",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_refresh_iot_session_falls_back_to_full_relogin() {
        super::timing::asynchronous(
            "aligned::aliyun_session_refresh::test_refresh_iot_session_falls_back_to_full_relogin",
            super::client_cases::run(
                "aliyun_session_refresh",
                "test_refresh_iot_session_falls_back_to_full_relogin",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_refresh_iot_session_skips_relogin_when_handshake_succeeds() {
        super::timing::asynchronous("aligned::aliyun_session_refresh::test_refresh_iot_session_skips_relogin_when_handshake_succeeds", super::client_cases::run("aliyun_session_refresh","test_refresh_iot_session_skips_relogin_when_handshake_succeeds")).await;
    }
}
mod aliyun_signing {
    #[test]
    fn test_headers_match_signature_fixture() {
        super::timing::sync(
            "aligned::aliyun_signing::test_headers_match_signature_fixture",
            || super::pure::run("aliyun_signing", "test_headers_match_signature_fixture"),
        );
    }
    #[test]
    fn test_empty_body_gets_md5_of_empty_string() {
        super::timing::sync(
            "aligned::aliyun_signing::test_empty_body_gets_md5_of_empty_string",
            || super::pure::run("aliyun_signing", "test_empty_body_gets_md5_of_empty_string"),
        );
    }
}
mod aliyun_transport {
    #[tokio::test]
    async fn test_call_posts_iot_envelope() {
        super::timing::asynchronous(
            "aligned::aliyun_transport::test_call_posts_iot_envelope",
            super::transport_cases::run("aliyun_transport", "test_call_posts_iot_envelope"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_call_oa_form_encodes_and_signs() {
        super::timing::asynchronous(
            "aligned::aliyun_transport::test_call_oa_form_encodes_and_signs",
            super::transport_cases::run("aliyun_transport", "test_call_oa_form_encodes_and_signs"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_post_error_status_raises_transport_error() {
        super::timing::asynchronous(
            "aligned::aliyun_transport::test_post_error_status_raises_transport_error",
            super::transport_cases::run(
                "aliyun_transport",
                "test_post_error_status_raises_transport_error",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_post_client_error_is_wrapped() {
        super::timing::asynchronous(
            "aligned::aliyun_transport::test_post_client_error_is_wrapped",
            super::transport_cases::run("aliyun_transport", "test_post_client_error_is_wrapped"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_post_timeout_is_wrapped() {
        super::timing::asynchronous(
            "aligned::aliyun_transport::test_post_timeout_is_wrapped",
            super::transport_cases::run("aliyun_transport", "test_post_timeout_is_wrapped"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_close_closes_owned_session() {
        super::timing::asynchronous(
            "aligned::aliyun_transport::test_close_closes_owned_session",
            super::transport_cases::run("aliyun_transport", "test_close_closes_owned_session"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_close_does_not_close_injected_session() {
        super::timing::asynchronous(
            "aligned::aliyun_transport::test_close_does_not_close_injected_session",
            super::transport_cases::run(
                "aliyun_transport",
                "test_close_does_not_close_injected_session",
            ),
        )
        .await;
    }
    #[test]
    fn test_encode_oa_body_url_encoded_vs_canonical() {
        super::timing::sync(
            "aligned::aliyun_transport::test_encode_oa_body_url_encoded_vs_canonical",
            || {
                super::pure::run(
                    "aliyun_transport",
                    "test_encode_oa_body_url_encoded_vs_canonical",
                )
            },
        );
    }
}
mod client_commands {
    #[tokio::test]
    async fn test_start_clean_invokes_clean_now_with_flag() {
        super::timing::asynchronous(
            "aligned::client_commands::test_start_clean_invokes_clean_now_with_flag",
            super::client_cases::run(
                "client_commands",
                "test_start_clean_invokes_clean_now_with_flag",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_stop_clean_sends_zero_flag() {
        super::timing::asynchronous(
            "aligned::client_commands::test_stop_clean_sends_zero_flag",
            super::client_cases::run("client_commands", "test_stop_clean_sends_zero_flag"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_start_level_invokes_sand_leveling() {
        super::timing::asynchronous(
            "aligned::client_commands::test_start_level_invokes_sand_leveling",
            super::client_cases::run("client_commands", "test_start_level_invokes_sand_leveling"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_stop_level_sends_zero_flag() {
        super::timing::asynchronous(
            "aligned::client_commands::test_stop_level_sends_zero_flag",
            super::client_cases::run("client_commands", "test_stop_level_sends_zero_flag"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_auto_level__605b69f1() {
        super::timing::asynchronous(
            "aligned::client_commands::test_set_auto_level__605b69f1",
            super::client_cases::run("client_commands", "test_set_auto_level[True-1]"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_auto_level__c0cbc821() {
        super::timing::asynchronous(
            "aligned::client_commands::test_set_auto_level__c0cbc821",
            super::client_cases::run("client_commands", "test_set_auto_level[False-0]"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_silent_mode__eb364819() {
        super::timing::asynchronous(
            "aligned::client_commands::test_set_silent_mode__eb364819",
            super::client_cases::run("client_commands", "test_set_silent_mode[True-1]"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_silent_mode__7b4a1847() {
        super::timing::asynchronous(
            "aligned::client_commands::test_set_silent_mode__7b4a1847",
            super::client_cases::run("client_commands", "test_set_silent_mode[False-0]"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_child_lock__3adabd6f() {
        super::timing::asynchronous(
            "aligned::client_commands::test_set_child_lock__3adabd6f",
            super::client_cases::run("client_commands", "test_set_child_lock[True-1]"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_child_lock__ca3fd25d() {
        super::timing::asynchronous(
            "aligned::client_commands::test_set_child_lock__ca3fd25d",
            super::client_cases::run("client_commands", "test_set_child_lock[False-0]"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_auto_clean_preserves_existing_mode() {
        super::timing::asynchronous(
            "aligned::client_commands::test_set_auto_clean_preserves_existing_mode",
            super::client_cases::run(
                "client_commands",
                "test_set_auto_clean_preserves_existing_mode",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_auto_clean_disable_sets_active_zero() {
        super::timing::asynchronous(
            "aligned::client_commands::test_set_auto_clean_disable_sets_active_zero",
            super::client_cases::run(
                "client_commands",
                "test_set_auto_clean_disable_sets_active_zero",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_calibrate_sand_valid() {
        super::timing::asynchronous(
            "aligned::client_commands::test_calibrate_sand_valid",
            super::client_cases::run("client_commands", "test_calibrate_sand_valid"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_calibrate_sand_out_of_range_raises_client_side__24504795() {
        super::timing::asynchronous("aligned::client_commands::test_calibrate_sand_out_of_range_raises_client_side__24504795", super::client_cases::run("client_commands","test_calibrate_sand_out_of_range_raises_client_side[0]")).await;
    }
    #[tokio::test]
    async fn test_calibrate_sand_out_of_range_raises_client_side__84a6caba() {
        super::timing::asynchronous("aligned::client_commands::test_calibrate_sand_out_of_range_raises_client_side__84a6caba", super::client_cases::run("client_commands","test_calibrate_sand_out_of_range_raises_client_side[101]")).await;
    }
    #[tokio::test]
    async fn test_calibrate_sand_out_of_range_raises_client_side__e5f86895() {
        super::timing::asynchronous("aligned::client_commands::test_calibrate_sand_out_of_range_raises_client_side__e5f86895", super::client_cases::run("client_commands","test_calibrate_sand_out_of_range_raises_client_side[-1]")).await;
    }
    #[tokio::test]
    async fn test_calibrate_sand_out_of_range_raises_client_side__35aa0468() {
        super::timing::asynchronous("aligned::client_commands::test_calibrate_sand_out_of_range_raises_client_side__35aa0468", super::client_cases::run("client_commands","test_calibrate_sand_out_of_range_raises_client_side[200]")).await;
    }
    #[tokio::test]
    async fn test_resolve_device_refreshes_index_on_miss() {
        super::timing::asynchronous(
            "aligned::client_commands::test_resolve_device_refreshes_index_on_miss",
            super::client_cases::run(
                "client_commands",
                "test_resolve_device_refreshes_index_on_miss",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_resolve_device_unknown_raises_after_refresh() {
        super::timing::asynchronous(
            "aligned::client_commands::test_resolve_device_unknown_raises_after_refresh",
            super::client_cases::run(
                "client_commands",
                "test_resolve_device_unknown_raises_after_refresh",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_resolve_role_returns_device_role() {
        super::timing::asynchronous(
            "aligned::client_commands::test_resolve_role_returns_device_role",
            super::client_cases::run("client_commands", "test_resolve_role_returns_device_role"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_get_status_unwraps_properties() {
        super::timing::asynchronous(
            "aligned::client_commands::test_get_status_unwraps_properties",
            super::client_cases::run("client_commands", "test_get_status_unwraps_properties"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_list_devices_builds_index() {
        super::timing::asynchronous(
            "aligned::client_commands::test_list_devices_builds_index",
            super::client_cases::run("client_commands", "test_list_devices_builds_index"),
        )
        .await;
    }
}
mod client_history {
    #[tokio::test]
    async fn test_get_toilet_records_signs_and_parses() {
        super::timing::asynchronous(
            "aligned::client_history::test_get_toilet_records_signs_and_parses",
            super::client_cases::run("client_history", "test_get_toilet_records_signs_and_parses"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_get_toilet_statistics_passes_zone() {
        super::timing::asynchronous(
            "aligned::client_history::test_get_toilet_statistics_passes_zone",
            super::client_cases::run("client_history", "test_get_toilet_statistics_passes_zone"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_list_cats_parses_array() {
        super::timing::asynchronous(
            "aligned::client_history::test_list_cats_parses_array",
            super::client_cases::run("client_history", "test_list_cats_parses_array"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_authenticated_get_stamps_user_id_and_unwraps() {
        super::timing::asynchronous(
            "aligned::client_history::test_authenticated_get_stamps_user_id_and_unwraps",
            super::client_cases::run(
                "client_history",
                "test_authenticated_get_stamps_user_id_and_unwraps",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_require_session_raises_before_login() {
        super::timing::asynchronous(
            "aligned::client_history::test_require_session_raises_before_login",
            super::client_cases::run("client_history", "test_require_session_raises_before_login"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_require_iot_session_raises_without_token() {
        super::timing::asynchronous(
            "aligned::client_history::test_require_iot_session_raises_without_token",
            super::client_cases::run(
                "client_history",
                "test_require_iot_session_raises_without_token",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_require_iot_session_returns_token() {
        super::timing::asynchronous(
            "aligned::client_history::test_require_iot_session_returns_token",
            super::client_cases::run("client_history", "test_require_iot_session_returns_token"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_watch_status_requires_login() {
        super::timing::asynchronous(
            "aligned::client_history::test_watch_status_requires_login",
            super::client_cases::run("client_history", "test_watch_status_requires_login"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_watch_status_returns_stream() {
        super::timing::asynchronous(
            "aligned::client_history::test_watch_status_returns_stream",
            super::client_cases::run("client_history", "test_watch_status_returns_stream"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_sends_only_weight_and_context__ab40d78a() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_sends_only_weight_and_context__ab40d78a",
            super::client_cases::run(
                "client_history",
                "test_set_cat_weight_sends_only_weight_and_context[4.8-kg-1]",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_sends_only_weight_and_context__b93bcd65() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_sends_only_weight_and_context__b93bcd65",
            super::client_cases::run(
                "client_history",
                "test_set_cat_weight_sends_only_weight_and_context[4.8-kg-2]",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_sends_only_weight_and_context__2ee0c48a() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_sends_only_weight_and_context__2ee0c48a",
            super::client_cases::run(
                "client_history",
                "test_set_cat_weight_sends_only_weight_and_context[10.58-lb-1]",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_sends_only_weight_and_context__71f3567d() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_sends_only_weight_and_context__71f3567d",
            super::client_cases::run(
                "client_history",
                "test_set_cat_weight_sends_only_weight_and_context[10.58-lb-2]",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_invalid_weight_before_io__a116e27c() {
        super::timing::asynchronous("aligned::client_history::test_set_cat_weight_rejects_invalid_weight_before_io__a116e27c", super::client_cases::run("client_history","test_set_cat_weight_rejects_invalid_weight_before_io[0]")).await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_invalid_weight_before_io__7e108401() {
        super::timing::asynchronous("aligned::client_history::test_set_cat_weight_rejects_invalid_weight_before_io__7e108401", super::client_cases::run("client_history","test_set_cat_weight_rejects_invalid_weight_before_io[-1]")).await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_invalid_weight_before_io__bddc8f3b() {
        super::timing::asynchronous("aligned::client_history::test_set_cat_weight_rejects_invalid_weight_before_io__bddc8f3b", super::client_cases::run("client_history","test_set_cat_weight_rejects_invalid_weight_before_io[nan]")).await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_invalid_weight_before_io__d7ed34e5() {
        super::timing::asynchronous("aligned::client_history::test_set_cat_weight_rejects_invalid_weight_before_io__d7ed34e5", super::client_cases::run("client_history","test_set_cat_weight_rejects_invalid_weight_before_io[inf]")).await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_invalid_weight_before_io__cec5065f() {
        super::timing::asynchronous("aligned::client_history::test_set_cat_weight_rejects_invalid_weight_before_io__cec5065f", super::client_cases::run("client_history","test_set_cat_weight_rejects_invalid_weight_before_io[-inf]")).await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_invalid_weight_before_io__8facc5a3() {
        super::timing::asynchronous("aligned::client_history::test_set_cat_weight_rejects_invalid_weight_before_io__8facc5a3", super::client_cases::run("client_history","test_set_cat_weight_rejects_invalid_weight_before_io[True]")).await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_invalid_cat_id__e4e59500() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_rejects_invalid_cat_id__e4e59500",
            super::client_cases::run(
                "client_history",
                "test_set_cat_weight_rejects_invalid_cat_id[0]",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_invalid_cat_id__d2201d6f() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_rejects_invalid_cat_id__d2201d6f",
            super::client_cases::run(
                "client_history",
                "test_set_cat_weight_rejects_invalid_cat_id[-1]",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_invalid_cat_id__74e4457b() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_rejects_invalid_cat_id__74e4457b",
            super::client_cases::run(
                "client_history",
                "test_set_cat_weight_rejects_invalid_cat_id[True]",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_rejects_unknown_unit() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_rejects_unknown_unit",
            super::client_cases::run("client_history", "test_set_cat_weight_rejects_unknown_unit"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_requires_login() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_requires_login",
            super::client_cases::run("client_history", "test_set_cat_weight_requires_login"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_set_cat_weight_does_not_retry_expired_session() {
        super::timing::asynchronous(
            "aligned::client_history::test_set_cat_weight_does_not_retry_expired_session",
            super::client_cases::run(
                "client_history",
                "test_set_cat_weight_does_not_retry_expired_session",
            ),
        )
        .await;
    }
}
mod client_lifecycle {
    #[tokio::test]
    async fn test_close_delegates_to_both_transports() {
        super::timing::asynchronous(
            "aligned::client_lifecycle::test_close_delegates_to_both_transports",
            super::client_cases::run(
                "client_lifecycle",
                "test_close_delegates_to_both_transports",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_async_context_manager_closes_on_exit() {
        super::timing::asynchronous(
            "aligned::client_lifecycle::test_async_context_manager_closes_on_exit",
            super::client_cases::run(
                "client_lifecycle",
                "test_async_context_manager_closes_on_exit",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_login_rest_parses_and_stores() {
        super::timing::asynchronous(
            "aligned::client_lifecycle::test_login_rest_parses_and_stores",
            super::client_cases::run("client_lifecycle", "test_login_rest_parses_and_stores"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_authenticate_aliyun_stores_iot_token() {
        super::timing::asynchronous(
            "aligned::client_lifecycle::test_authenticate_aliyun_stores_iot_token",
            super::client_cases::run(
                "client_lifecycle",
                "test_authenticate_aliyun_stores_iot_token",
            ),
        )
        .await;
    }
    #[test]
    fn test_expect_object_passes_through_mapping() {
        super::timing::sync(
            "aligned::client_lifecycle::test_expect_object_passes_through_mapping",
            || {
                super::pure::run(
                    "client_lifecycle",
                    "test_expect_object_passes_through_mapping",
                )
            },
        );
    }
    #[test]
    fn test_expect_object_rejects_non_object__bbf4974d() {
        super::timing::sync(
            "aligned::client_lifecycle::test_expect_object_rejects_non_object__bbf4974d",
            || {
                super::pure::run(
                    "client_lifecycle",
                    "test_expect_object_rejects_non_object[value0]",
                )
            },
        );
    }
    #[test]
    fn test_expect_object_rejects_non_object__bfd8eb0d() {
        super::timing::sync(
            "aligned::client_lifecycle::test_expect_object_rejects_non_object__bfd8eb0d",
            || {
                super::pure::run(
                    "client_lifecycle",
                    "test_expect_object_rejects_non_object[str]",
                )
            },
        );
    }
    #[test]
    fn test_expect_object_rejects_non_object__67fa0e21() {
        super::timing::sync(
            "aligned::client_lifecycle::test_expect_object_rejects_non_object__67fa0e21",
            || {
                super::pure::run(
                    "client_lifecycle",
                    "test_expect_object_rejects_non_object[5]",
                )
            },
        );
    }
    #[test]
    fn test_expect_object_rejects_non_object__17c75186() {
        super::timing::sync(
            "aligned::client_lifecycle::test_expect_object_rejects_non_object__17c75186",
            || {
                super::pure::run(
                    "client_lifecycle",
                    "test_expect_object_rejects_non_object[None]",
                )
            },
        );
    }
}
mod client_login {
    #[tokio::test]
    async fn test_login_fresh_runs_rest_then_handshake() {
        super::timing::asynchronous(
            "aligned::client_login::test_login_fresh_runs_rest_then_handshake",
            super::client_cases::run("client_login", "test_login_fresh_runs_rest_then_handshake"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_login_cached_with_iot_token_mints_nothing() {
        super::timing::asynchronous(
            "aligned::client_login::test_login_cached_with_iot_token_mints_nothing",
            super::client_cases::run(
                "client_login",
                "test_login_cached_with_iot_token_mints_nothing",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_login_cached_with_iot_token_restores_cached_aliyun_host() {
        super::timing::asynchronous(
            "aligned::client_login::test_login_cached_with_iot_token_restores_cached_aliyun_host",
            super::client_cases::run(
                "client_login",
                "test_login_cached_with_iot_token_restores_cached_aliyun_host",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_login_cached_without_iot_token_only_handshakes() {
        super::timing::asynchronous(
            "aligned::client_login::test_login_cached_without_iot_token_only_handshakes",
            super::client_cases::run(
                "client_login",
                "test_login_cached_without_iot_token_only_handshakes",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_region_property() {
        super::timing::asynchronous(
            "aligned::client_login::test_region_property",
            super::client_cases::run("client_login", "test_region_property"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_login_result_property_starts_none() {
        super::timing::asynchronous(
            "aligned::client_login::test_login_result_property_starts_none",
            super::client_cases::run("client_login", "test_login_result_property_starts_none"),
        )
        .await;
    }
    #[test]
    fn test_user_agent_tracks_installed_version() {
        super::timing::sync(
            "aligned::client_login::test_user_agent_tracks_installed_version",
            || super::pure::run("client_login", "test_user_agent_tracks_installed_version"),
        );
    }
    #[tokio::test]
    async fn test_login_params_carry_sdk_user_agent_as_dev_sys_ver() {
        super::timing::asynchronous(
            "aligned::client_login::test_login_params_carry_sdk_user_agent_as_dev_sys_ver",
            super::client_cases::run(
                "client_login",
                "test_login_params_carry_sdk_user_agent_as_dev_sys_ver",
            ),
        )
        .await;
    }
    #[test]
    fn test_sdk_version_falls_back_when_package_metadata_is_missing() {
        super::timing::sync(
            "aligned::client_login::test_sdk_version_falls_back_when_package_metadata_is_missing",
            || {
                super::pure::run(
                    "client_login",
                    "test_sdk_version_falls_back_when_package_metadata_is_missing",
                )
            },
        );
    }
}
mod error_dispatch {
    #[test]
    fn test_zero_code_returns_data() {
        super::timing::sync(
            "aligned::error_dispatch::test_zero_code_returns_data",
            || super::pure::run("error_dispatch", "test_zero_code_returns_data"),
        );
    }
    #[test]
    fn test_session_expired_codes_dispatch_to_session_expired_error__f41401be() {
        super::timing::sync(
            "aligned::error_dispatch::test_session_expired_codes_dispatch_to_session_expired_error__f41401be",
            || {
                super::pure::run(
                    "error_dispatch",
                    "test_session_expired_codes_dispatch_to_session_expired_error[1007]",
                )
            },
        );
    }
    #[test]
    fn test_session_expired_codes_dispatch_to_session_expired_error__0477beac() {
        super::timing::sync(
            "aligned::error_dispatch::test_session_expired_codes_dispatch_to_session_expired_error__0477beac",
            || {
                super::pure::run(
                    "error_dispatch",
                    "test_session_expired_codes_dispatch_to_session_expired_error[3026]",
                )
            },
        );
    }
    #[test]
    fn test_session_expired_codes_dispatch_to_session_expired_error__ed91e884() {
        super::timing::sync(
            "aligned::error_dispatch::test_session_expired_codes_dispatch_to_session_expired_error__ed91e884",
            || {
                super::pure::run(
                    "error_dispatch",
                    "test_session_expired_codes_dispatch_to_session_expired_error[3027]",
                )
            },
        );
    }
    #[test]
    fn test_invalid_credentials_codes_dispatch_to_invalid_credentials_error__e1106651() {
        super::timing::sync(
            "aligned::error_dispatch::test_invalid_credentials_codes_dispatch_to_invalid_credentials_error__e1106651",
            || {
                super::pure::run(
                    "error_dispatch",
                    "test_invalid_credentials_codes_dispatch_to_invalid_credentials_error[3019]",
                )
            },
        );
    }
    #[test]
    fn test_invalid_credentials_codes_dispatch_to_invalid_credentials_error__c3146d0d() {
        super::timing::sync(
            "aligned::error_dispatch::test_invalid_credentials_codes_dispatch_to_invalid_credentials_error__c3146d0d",
            || {
                super::pure::run(
                    "error_dispatch",
                    "test_invalid_credentials_codes_dispatch_to_invalid_credentials_error[10060]",
                )
            },
        );
    }
    #[test]
    fn test_invalid_credentials_codes_dispatch_to_invalid_credentials_error__729e67ad() {
        super::timing::sync(
            "aligned::error_dispatch::test_invalid_credentials_codes_dispatch_to_invalid_credentials_error__729e67ad",
            || {
                super::pure::run(
                    "error_dispatch",
                    "test_invalid_credentials_codes_dispatch_to_invalid_credentials_error[10061]",
                )
            },
        );
    }
    #[test]
    fn test_invalid_credentials_codes_dispatch_to_invalid_credentials_error__0d2e04f8() {
        super::timing::sync(
            "aligned::error_dispatch::test_invalid_credentials_codes_dispatch_to_invalid_credentials_error__0d2e04f8",
            || {
                super::pure::run(
                    "error_dispatch",
                    "test_invalid_credentials_codes_dispatch_to_invalid_credentials_error[10192]",
                )
            },
        );
    }
    #[test]
    fn test_unknown_auth_code_falls_back_to_authentication_error() {
        super::timing::sync(
            "aligned::error_dispatch::test_unknown_auth_code_falls_back_to_authentication_error",
            || {
                super::pure::run(
                    "error_dispatch",
                    "test_unknown_auth_code_falls_back_to_authentication_error",
                )
            },
        );
    }
    #[test]
    fn test_non_auth_context_uses_plain_api_error() {
        super::timing::sync(
            "aligned::error_dispatch::test_non_auth_context_uses_plain_api_error",
            || {
                super::pure::run(
                    "error_dispatch",
                    "test_non_auth_context_uses_plain_api_error",
                )
            },
        );
    }
}
mod eu_gateway_integration {
    #[tokio::test]
    async fn test_eu_device_call_routes_through_regional_gateway() {
        super::timing::asynchronous(
            "aligned::eu_gateway_integration::test_eu_device_call_routes_through_regional_gateway",
            super::client_cases::run(
                "eu_gateway_integration",
                "test_eu_device_call_routes_through_regional_gateway",
            ),
        )
        .await;
    }
}
mod http_transport {
    #[tokio::test]
    async fn test_signed_get_decodes_json_object() {
        super::timing::asynchronous(
            "aligned::http_transport::test_signed_get_decodes_json_object",
            super::transport_cases::run("http_transport", "test_signed_get_decodes_json_object"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_signed_get_data_envelope_wraps_params() {
        super::timing::asynchronous(
            "aligned::http_transport::test_signed_get_data_envelope_wraps_params",
            super::transport_cases::run(
                "http_transport",
                "test_signed_get_data_envelope_wraps_params",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_authenticated_get_sets_uid_and_token_headers() {
        super::timing::asynchronous(
            "aligned::http_transport::test_authenticated_get_sets_uid_and_token_headers",
            super::transport_cases::run(
                "http_transport",
                "test_authenticated_get_sets_uid_and_token_headers",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_http_error_status_raises_transport_error() {
        super::timing::asynchronous(
            "aligned::http_transport::test_http_error_status_raises_transport_error",
            super::transport_cases::run(
                "http_transport",
                "test_http_error_status_raises_transport_error",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_client_error_is_wrapped() {
        super::timing::asynchronous(
            "aligned::http_transport::test_client_error_is_wrapped",
            super::transport_cases::run("http_transport", "test_client_error_is_wrapped"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_timeout_is_wrapped() {
        super::timing::asynchronous(
            "aligned::http_transport::test_timeout_is_wrapped",
            super::transport_cases::run("http_transport", "test_timeout_is_wrapped"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_close_closes_owned_session() {
        super::timing::asynchronous(
            "aligned::http_transport::test_close_closes_owned_session",
            super::transport_cases::run("http_transport", "test_close_closes_owned_session"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_close_does_not_close_injected_session() {
        super::timing::asynchronous(
            "aligned::http_transport::test_close_does_not_close_injected_session",
            super::transport_cases::run(
                "http_transport",
                "test_close_does_not_close_injected_session",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_close_is_idempotent_when_never_used() {
        super::timing::asynchronous(
            "aligned::http_transport::test_close_is_idempotent_when_never_used",
            super::transport_cases::run(
                "http_transport",
                "test_close_is_idempotent_when_never_used",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_authenticated_post_sends_json_and_session_headers() {
        super::timing::asynchronous(
            "aligned::http_transport::test_authenticated_post_sends_json_and_session_headers",
            super::transport_cases::run(
                "http_transport",
                "test_authenticated_post_sends_json_and_session_headers",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_authenticated_post_wraps_network_errors__23d32649() {
        super::timing::asynchronous(
            "aligned::http_transport::test_authenticated_post_wraps_network_errors__23d32649",
            super::transport_cases::run(
                "http_transport",
                "test_authenticated_post_wraps_network_errors[error0]",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_authenticated_post_wraps_network_errors__f1d16769() {
        super::timing::asynchronous(
            "aligned::http_transport::test_authenticated_post_wraps_network_errors__f1d16769",
            super::transport_cases::run(
                "http_transport",
                "test_authenticated_post_wraps_network_errors[error1]",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_authenticated_post_http_error() {
        super::timing::asynchronous(
            "aligned::http_transport::test_authenticated_post_http_error",
            super::transport_cases::run("http_transport", "test_authenticated_post_http_error"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_authenticated_post_creates_and_closes_owned_session() {
        super::timing::asynchronous(
            "aligned::http_transport::test_authenticated_post_creates_and_closes_owned_session",
            super::transport_cases::run(
                "http_transport",
                "test_authenticated_post_creates_and_closes_owned_session",
            ),
        )
        .await;
    }
}
mod models {
    #[test]
    fn test_cat_parses_live_sample() {
        super::timing::sync("aligned::models::test_cat_parses_live_sample", || {
            super::pure::run("models", "test_cat_parses_live_sample")
        });
    }
    #[test]
    fn test_cat_unknown_gender_falls_back() {
        super::timing::sync(
            "aligned::models::test_cat_unknown_gender_falls_back",
            || super::pure::run("models", "test_cat_unknown_gender_falls_back"),
        );
    }
    #[test]
    fn test_cat_list_from_response_filters_non_objects() {
        super::timing::sync(
            "aligned::models::test_cat_list_from_response_filters_non_objects",
            || super::pure::run("models", "test_cat_list_from_response_filters_non_objects"),
        );
    }
    #[test]
    fn test_cat_list_from_response_handles_unexpected_top_level() {
        super::timing::sync(
            "aligned::models::test_cat_list_from_response_handles_unexpected_top_level",
            || {
                super::pure::run(
                    "models",
                    "test_cat_list_from_response_handles_unexpected_top_level",
                )
            },
        );
    }
    #[test]
    fn test_toilet_record_visit_sample() {
        super::timing::sync("aligned::models::test_toilet_record_visit_sample", || {
            super::pure::run("models", "test_toilet_record_visit_sample")
        });
    }
    #[test]
    fn test_toilet_record_clean_cycle_sample() {
        super::timing::sync(
            "aligned::models::test_toilet_record_clean_cycle_sample",
            || super::pure::run("models", "test_toilet_record_clean_cycle_sample"),
        );
    }
    #[test]
    fn test_toilet_record_list_from_response_unwraps_envelope() {
        super::timing::sync(
            "aligned::models::test_toilet_record_list_from_response_unwraps_envelope",
            || {
                super::pure::run(
                    "models",
                    "test_toilet_record_list_from_response_unwraps_envelope",
                )
            },
        );
    }
    #[test]
    fn test_toilet_record_list_from_response_tolerates_missing_envelope() {
        super::timing::sync(
            "aligned::models::test_toilet_record_list_from_response_tolerates_missing_envelope",
            || {
                super::pure::run(
                    "models",
                    "test_toilet_record_list_from_response_tolerates_missing_envelope",
                )
            },
        );
    }
    #[test]
    fn test_device_status_bucket_full_reads_room_of_bin() {
        super::timing::sync(
            "aligned::models::test_device_status_bucket_full_reads_room_of_bin",
            || super::pure::run("models", "test_device_status_bucket_full_reads_room_of_bin"),
        );
    }
    #[test]
    fn test_device_status_operating_state_reads_bucket_status() {
        super::timing::sync(
            "aligned::models::test_device_status_operating_state_reads_bucket_status",
            || {
                super::pure::run(
                    "models",
                    "test_device_status_operating_state_reads_bucket_status",
                )
            },
        );
    }
    #[test]
    fn test_operating_state_from_code() {
        super::timing::sync("aligned::models::test_operating_state_from_code", || {
            super::pure::run("models", "test_operating_state_from_code")
        });
    }
    #[test]
    fn test_operating_state_unmapped_code_warns_once() {
        super::timing::sync(
            "aligned::models::test_operating_state_unmapped_code_warns_once",
            || super::pure::run("models", "test_operating_state_unmapped_code_warns_once"),
        );
    }
    #[test]
    fn test_daily_statistics_parses_live_sample() {
        super::timing::sync(
            "aligned::models::test_daily_statistics_parses_live_sample",
            || super::pure::run("models", "test_daily_statistics_parses_live_sample"),
        );
    }
    #[test]
    fn test_daily_statistics_list_from_response() {
        super::timing::sync(
            "aligned::models::test_daily_statistics_list_from_response",
            || super::pure::run("models", "test_daily_statistics_list_from_response"),
        );
    }
}
mod models_extra {
    #[test]
    fn test_user_info_round_trips_through_dict() {
        super::timing::sync(
            "aligned::models_extra::test_user_info_round_trips_through_dict",
            || super::pure::run("models_extra", "test_user_info_round_trips_through_dict"),
        );
    }
    #[test]
    fn test_login_result_round_trips_through_dict() {
        super::timing::sync(
            "aligned::models_extra::test_login_result_round_trips_through_dict",
            || super::pure::run("models_extra", "test_login_result_round_trips_through_dict"),
        );
    }
    #[test]
    fn test_login_result_round_trips_non_default_iot_host() {
        super::timing::sync(
            "aligned::models_extra::test_login_result_round_trips_non_default_iot_host",
            || {
                super::pure::run(
                    "models_extra",
                    "test_login_result_round_trips_non_default_iot_host",
                )
            },
        );
    }
    #[test]
    fn test_login_result_from_dict_defaults_iot_host_for_legacy_cache() {
        super::timing::sync(
            "aligned::models_extra::test_login_result_from_dict_defaults_iot_host_for_legacy_cache",
            || {
                super::pure::run(
                    "models_extra",
                    "test_login_result_from_dict_defaults_iot_host_for_legacy_cache",
                )
            },
        );
    }
    #[test]
    fn test_login_result_age_seconds() {
        super::timing::sync(
            "aligned::models_extra::test_login_result_age_seconds",
            || super::pure::run("models_extra", "test_login_result_age_seconds"),
        );
    }
    #[test]
    fn test_parse_login_token_empty_returns_blanks() {
        super::timing::sync(
            "aligned::models_extra::test_parse_login_token_empty_returns_blanks",
            || {
                super::pure::run(
                    "models_extra",
                    "test_parse_login_token_empty_returns_blanks",
                )
            },
        );
    }
    #[test]
    fn test_parse_login_token_decrypts_and_splits() {
        super::timing::sync(
            "aligned::models_extra::test_parse_login_token_decrypts_and_splits",
            || super::pure::run("models_extra", "test_parse_login_token_decrypts_and_splits"),
        );
    }
    #[test]
    fn test_parse_login_token_pads_short_token() {
        super::timing::sync(
            "aligned::models_extra::test_parse_login_token_pads_short_token",
            || super::pure::run("models_extra", "test_parse_login_token_pads_short_token"),
        );
    }
    #[test]
    fn test_login_result_from_json_prefers_ali_user_id() {
        super::timing::sync(
            "aligned::models_extra::test_login_result_from_json_prefers_ali_user_id",
            || {
                super::pure::run(
                    "models_extra",
                    "test_login_result_from_json_prefers_ali_user_id",
                )
            },
        );
    }
    #[test]
    fn test_login_result_from_json_falls_back_to_embedded_user_id() {
        super::timing::sync(
            "aligned::models_extra::test_login_result_from_json_falls_back_to_embedded_user_id",
            || {
                super::pure::run(
                    "models_extra",
                    "test_login_result_from_json_falls_back_to_embedded_user_id",
                )
            },
        );
    }
    #[test]
    fn test_device_list_from_response_ignores_non_array() {
        super::timing::sync(
            "aligned::models_extra::test_device_list_from_response_ignores_non_array",
            || {
                super::pure::run(
                    "models_extra",
                    "test_device_list_from_response_ignores_non_array",
                )
            },
        );
    }
    #[test]
    fn test_device_list_from_response_skips_non_mapping_entries() {
        super::timing::sync(
            "aligned::models_extra::test_device_list_from_response_skips_non_mapping_entries",
            || {
                super::pure::run(
                    "models_extra",
                    "test_device_list_from_response_skips_non_mapping_entries",
                )
            },
        );
    }
    #[test]
    fn test_record_type_unknown_falls_back_to_other() {
        super::timing::sync(
            "aligned::models_extra::test_record_type_unknown_falls_back_to_other",
            || {
                super::pure::run(
                    "models_extra",
                    "test_record_type_unknown_falls_back_to_other",
                )
            },
        );
    }
    #[test]
    fn test_device_status_property_int_coerces_numeric_string() {
        super::timing::sync(
            "aligned::models_extra::test_device_status_property_int_coerces_numeric_string",
            || {
                super::pure::run(
                    "models_extra",
                    "test_device_status_property_int_coerces_numeric_string",
                )
            },
        );
    }
    #[test]
    fn test_device_status_property_int_non_numeric_string_falls_back_to_zero() {
        super::timing::sync(
            "aligned::models_extra::test_device_status_property_int_non_numeric_string_falls_back_to_zero",
            || {
                super::pure::run(
                    "models_extra",
                    "test_device_status_property_int_non_numeric_string_falls_back_to_zero",
                )
            },
        );
    }
    #[test]
    fn test_loads_rejects_non_object_top_level() {
        super::timing::sync(
            "aligned::models_extra::test_loads_rejects_non_object_top_level",
            || super::pure::run("models_extra", "test_loads_rejects_non_object_top_level"),
        );
    }
    #[test]
    fn test_get_str_default_on_wrong_type() {
        super::timing::sync(
            "aligned::models_extra::test_get_str_default_on_wrong_type",
            || super::pure::run("models_extra", "test_get_str_default_on_wrong_type"),
        );
    }
    #[test]
    fn test_get_int_coerces_and_falls_back() {
        super::timing::sync(
            "aligned::models_extra::test_get_int_coerces_and_falls_back",
            || super::pure::run("models_extra", "test_get_int_coerces_and_falls_back"),
        );
    }
    #[test]
    fn test_get_float_coerces_and_falls_back() {
        super::timing::sync(
            "aligned::models_extra::test_get_float_coerces_and_falls_back",
            || super::pure::run("models_extra", "test_get_float_coerces_and_falls_back"),
        );
    }
    #[test]
    fn test_get_object_returns_empty_on_non_mapping() {
        super::timing::sync(
            "aligned::models_extra::test_get_object_returns_empty_on_non_mapping",
            || {
                super::pure::run(
                    "models_extra",
                    "test_get_object_returns_empty_on_non_mapping",
                )
            },
        );
    }
}
mod mqtt_auth {
    #[tokio::test]
    async fn test_authinfo_sign_is_hex_lower_hmac_sha1() {
        super::timing::asynchronous(
            "aligned::mqtt_auth::test_authinfo_sign_is_hex_lower_hmac_sha1",
            super::stream_cases::run("mqtt_auth", "test_authinfo_sign_is_hex_lower_hmac_sha1"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_authinfo_random_values_are_well_formed() {
        super::timing::asynchronous(
            "aligned::mqtt_auth::test_authinfo_random_values_are_well_formed",
            super::stream_cases::run("mqtt_auth", "test_authinfo_random_values_are_well_formed"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_mqtt_password_sign() {
        super::timing::asynchronous(
            "aligned::mqtt_auth::test_mqtt_password_sign",
            super::stream_cases::run("mqtt_auth", "test_mqtt_password_sign"),
        )
        .await;
    }
}
mod mqtt_auth_derive {
    #[tokio::test]
    async fn test_derive_assembles_client_id_username_password() {
        super::timing::asynchronous(
            "aligned::mqtt_auth_derive::test_derive_assembles_client_id_username_password",
            super::stream_cases::run(
                "mqtt_auth_derive",
                "test_derive_assembles_client_id_username_password",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_derive_defaults_to_us_gateway() {
        super::timing::asynchronous(
            "aligned::mqtt_auth_derive::test_derive_defaults_to_us_gateway",
            super::stream_cases::run("mqtt_auth_derive", "test_derive_defaults_to_us_gateway"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_derive_follows_regional_gateway() {
        super::timing::asynchronous(
            "aligned::mqtt_auth_derive::test_derive_follows_regional_gateway",
            super::stream_cases::run("mqtt_auth_derive", "test_derive_follows_regional_gateway"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_derive_raises_on_non_200_code() {
        super::timing::asynchronous(
            "aligned::mqtt_auth_derive::test_derive_raises_on_non_200_code",
            super::stream_cases::run("mqtt_auth_derive", "test_derive_raises_on_non_200_code"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_derive_raises_when_data_not_object() {
        super::timing::asynchronous(
            "aligned::mqtt_auth_derive::test_derive_raises_when_data_not_object",
            super::stream_cases::run(
                "mqtt_auth_derive",
                "test_derive_raises_when_data_not_object",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_derive_raises_on_empty_triple_fields() {
        super::timing::asynchronous(
            "aligned::mqtt_auth_derive::test_derive_raises_on_empty_triple_fields",
            super::stream_cases::run(
                "mqtt_auth_derive",
                "test_derive_raises_on_empty_triple_fields",
            ),
        )
        .await;
    }
}
mod mqtt_transport {
    #[tokio::test]
    async fn test_build_tls_context_always_validates() {
        super::timing::asynchronous(
            "aligned::mqtt_transport::test_build_tls_context_always_validates",
            super::stream_cases::run("mqtt_transport", "test_build_tls_context_always_validates"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_build_tls_context_trusts_bundled_broker_root() {
        super::timing::asynchronous(
            "aligned::mqtt_transport::test_build_tls_context_trusts_bundled_broker_root",
            super::stream_cases::run(
                "mqtt_transport",
                "test_build_tls_context_trusts_bundled_broker_root",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_build_tls_context_keeps_broker_root_alongside_custom_bundle() {
        super::timing::asynchronous("aligned::mqtt_transport::test_build_tls_context_keeps_broker_root_alongside_custom_bundle", super::stream_cases::run("mqtt_transport","test_build_tls_context_keeps_broker_root_alongside_custom_bundle")).await;
    }
    #[tokio::test]
    async fn test_init_does_not_build_context_eagerly() {
        super::timing::asynchronous(
            "aligned::mqtt_transport::test_init_does_not_build_context_eagerly",
            super::stream_cases::run("mqtt_transport", "test_init_does_not_build_context_eagerly"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_init_accepts_prebuilt_context() {
        super::timing::asynchronous(
            "aligned::mqtt_transport::test_init_accepts_prebuilt_context",
            super::stream_cases::run("mqtt_transport", "test_init_accepts_prebuilt_context"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_connect_builds_context_in_executor() {
        super::timing::asynchronous(
            "aligned::mqtt_transport::test_connect_builds_context_in_executor",
            super::stream_cases::run("mqtt_transport", "test_connect_builds_context_in_executor"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_connect_uses_prebuilt_context() {
        super::timing::asynchronous(
            "aligned::mqtt_transport::test_connect_uses_prebuilt_context",
            super::stream_cases::run("mqtt_transport", "test_connect_uses_prebuilt_context"),
        )
        .await;
    }
}
mod mqtt_transport_flow {
    #[tokio::test]
    async fn test_topic_prefix() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_topic_prefix",
            super::stream_cases::run("mqtt_transport_flow", "test_topic_prefix"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_require_client_raises_when_disconnected() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_require_client_raises_when_disconnected",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_require_client_raises_when_disconnected",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_subscribe_and_publish_delegate_to_client() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_subscribe_and_publish_delegate_to_client",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_subscribe_and_publish_delegate_to_client",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_bind_account_success_resolves_on_matching_reply() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_bind_account_success_resolves_on_matching_reply",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_bind_account_success_resolves_on_matching_reply",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_bind_account_raises_on_non_200_reply() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_bind_account_raises_on_non_200_reply",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_bind_account_raises_on_non_200_reply",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_bind_account_times_out() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_bind_account_times_out",
            super::stream_cases::run("mqtt_transport_flow", "test_bind_account_times_out"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_try_dispatch_reply_routes_matching_id() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_try_dispatch_reply_routes_matching_id",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_try_dispatch_reply_routes_matching_id",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_try_dispatch_reply_ignores_unparsable_or_unmatched() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_try_dispatch_reply_ignores_unparsable_or_unmatched",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_try_dispatch_reply_ignores_unparsable_or_unmatched",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_try_dispatch_reply_ignores_already_done_future() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_try_dispatch_reply_ignores_already_done_future",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_try_dispatch_reply_ignores_already_done_future",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_disconnect_cancels_task_and_exits_client() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_disconnect_cancels_task_and_exits_client",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_disconnect_cancels_task_and_exits_client",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_disconnect_is_idempotent() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_disconnect_is_idempotent",
            super::stream_cases::run("mqtt_transport_flow", "test_disconnect_is_idempotent"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_disconnect_suppresses_mqtt_error_on_exit() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_disconnect_suppresses_mqtt_error_on_exit",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_disconnect_suppresses_mqtt_error_on_exit",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_connect_is_noop_when_already_connected() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_connect_is_noop_when_already_connected",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_connect_is_noop_when_already_connected",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_dispatch_loop_propagates_cancellation() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_dispatch_loop_propagates_cancellation",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_dispatch_loop_propagates_cancellation",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_dispatch_loop_routes_property_message_to_handler() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_dispatch_loop_routes_property_message_to_handler",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_dispatch_loop_routes_property_message_to_handler",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_dispatch_loop_routes_reply_before_handler() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_dispatch_loop_routes_reply_before_handler",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_dispatch_loop_routes_reply_before_handler",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_dispatch_loop_swallows_handler_exception() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_dispatch_loop_swallows_handler_exception",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_dispatch_loop_swallows_handler_exception",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_dispatch_loop_logs_and_exits_on_mqtt_error() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_dispatch_loop_logs_and_exits_on_mqtt_error",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_dispatch_loop_logs_and_exits_on_mqtt_error",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_dispatch_loop_signals_connection_lost_on_mqtt_error() {
        super::timing::asynchronous("aligned::mqtt_transport_flow::test_dispatch_loop_signals_connection_lost_on_mqtt_error", super::stream_cases::run("mqtt_transport_flow","test_dispatch_loop_signals_connection_lost_on_mqtt_error")).await;
    }
    #[tokio::test]
    async fn test_dispatch_loop_signals_connection_lost_when_stream_ends() {
        super::timing::asynchronous("aligned::mqtt_transport_flow::test_dispatch_loop_signals_connection_lost_when_stream_ends", super::stream_cases::run("mqtt_transport_flow","test_dispatch_loop_signals_connection_lost_when_stream_ends")).await;
    }
    #[tokio::test]
    async fn test_dispatch_loop_does_not_signal_on_cancellation() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_dispatch_loop_does_not_signal_on_cancellation",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_dispatch_loop_does_not_signal_on_cancellation",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_dispatch_loop_converts_bytearray_payload() {
        super::timing::asynchronous(
            "aligned::mqtt_transport_flow::test_dispatch_loop_converts_bytearray_payload",
            super::stream_cases::run(
                "mqtt_transport_flow",
                "test_dispatch_loop_converts_bytearray_payload",
            ),
        )
        .await;
    }
}
mod oa_signing {
    #[test]
    fn test_oa_signing_canonical_string() {
        super::timing::sync(
            "aligned::oa_signing::test_oa_signing_canonical_string",
            || super::pure::run("oa_signing", "test_oa_signing_canonical_string"),
        );
    }
    #[test]
    fn test_oa_body_wire_form_url_encodes_json_values() {
        super::timing::sync(
            "aligned::oa_signing::test_oa_body_wire_form_url_encodes_json_values",
            || {
                super::pure::run(
                    "oa_signing",
                    "test_oa_body_wire_form_url_encodes_json_values",
                )
            },
        );
    }
}
mod packaging_config {
    #[test]
    fn test_requires_python_is_the_lowest_supported_version() {
        super::timing::sync(
            "aligned::packaging_config::test_requires_python_is_the_lowest_supported_version",
            || {
                super::pure::run(
                    "packaging_config",
                    "test_requires_python_is_the_lowest_supported_version",
                )
            },
        );
    }
    #[test]
    fn test_classifiers_list_every_supported_version() {
        super::timing::sync(
            "aligned::packaging_config::test_classifiers_list_every_supported_version",
            || {
                super::pure::run(
                    "packaging_config",
                    "test_classifiers_list_every_supported_version",
                )
            },
        );
    }
    #[test]
    fn test_ruff_targets_the_minimum_supported_version() {
        super::timing::sync(
            "aligned::packaging_config::test_ruff_targets_the_minimum_supported_version",
            || {
                super::pure::run(
                    "packaging_config",
                    "test_ruff_targets_the_minimum_supported_version",
                )
            },
        );
    }
    #[test]
    fn test_mypy_targets_the_minimum_supported_version() {
        super::timing::sync(
            "aligned::packaging_config::test_mypy_targets_the_minimum_supported_version",
            || {
                super::pure::run(
                    "packaging_config",
                    "test_mypy_targets_the_minimum_supported_version",
                )
            },
        );
    }
}
mod region {
    #[test]
    fn test_region_web_url__e54079f7() {
        super::timing::sync("aligned::region::test_region_web_url__e54079f7", || {
            super::pure::run(
                "region",
                "test_region_web_url[Region.US-https://us.neakasa.com/api]",
            )
        });
    }
    #[test]
    fn test_region_web_url__b0bc1a01() {
        super::timing::sync("aligned::region::test_region_web_url__b0bc1a01", || {
            super::pure::run(
                "region",
                "test_region_web_url[Region.EU-https://euapi.neakasa.com/api]",
            )
        });
    }
    #[test]
    fn test_region_web_url__25e57719() {
        super::timing::sync("aligned::region::test_region_web_url__25e57719", || {
            super::pure::run(
                "region",
                "test_region_web_url[Region.AP-https://ap.neakasa.com/api]",
            )
        });
    }
    #[test]
    fn test_region_login_url_appends_login() {
        super::timing::sync(
            "aligned::region::test_region_login_url_appends_login",
            || super::pure::run("region", "test_region_login_url_appends_login"),
        );
    }
    #[test]
    fn test_eu_host_is_not_the_storefront_subdomain() {
        super::timing::sync(
            "aligned::region::test_eu_host_is_not_the_storefront_subdomain",
            || super::pure::run("region", "test_eu_host_is_not_the_storefront_subdomain"),
        );
    }
}
mod session_crypto {
    #[test]
    fn test_boot_key_encrypt_decrypt_roundtrip() {
        super::timing::sync(
            "aligned::session_crypto::test_boot_key_encrypt_decrypt_roundtrip",
            || super::pure::run("session_crypto", "test_boot_key_encrypt_decrypt_roundtrip"),
        );
    }
    #[test]
    fn test_aes_encrypt_pads_with_nul_to_block_size() {
        super::timing::sync(
            "aligned::session_crypto::test_aes_encrypt_pads_with_nul_to_block_size",
            || {
                super::pure::run(
                    "session_crypto",
                    "test_aes_encrypt_pads_with_nul_to_block_size",
                )
            },
        );
    }
    #[test]
    fn test_aes_encrypt_with_custom_key_roundtrip() {
        super::timing::sync(
            "aligned::session_crypto::test_aes_encrypt_with_custom_key_roundtrip",
            || {
                super::pure::run(
                    "session_crypto",
                    "test_aes_encrypt_with_custom_key_roundtrip",
                )
            },
        );
    }
    #[test]
    fn test_session_token_pins_against_known_inputs() {
        super::timing::sync(
            "aligned::session_crypto::test_session_token_pins_against_known_inputs",
            || {
                super::pure::run(
                    "session_crypto",
                    "test_session_token_pins_against_known_inputs",
                )
            },
        );
    }
}
mod status_stream {
    #[tokio::test]
    async fn test_start_connects_subscribes_binds() {
        super::timing::asynchronous(
            "aligned::status_stream::test_start_connects_subscribes_binds",
            super::stream_cases::run("status_stream", "test_start_connects_subscribes_binds"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_stop_disconnects_and_is_idempotent() {
        super::timing::asynchronous(
            "aligned::status_stream::test_stop_disconnects_and_is_idempotent",
            super::stream_cases::run("status_stream", "test_stop_disconnects_and_is_idempotent"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_run_forever_returns_after_stop() {
        super::timing::asynchronous(
            "aligned::status_stream::test_run_forever_returns_after_stop",
            super::stream_cases::run("status_stream", "test_run_forever_returns_after_stop"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_start_disconnects_when_subscribe_fails() {
        super::timing::asynchronous(
            "aligned::status_stream::test_start_disconnects_when_subscribe_fails",
            super::stream_cases::run(
                "status_stream",
                "test_start_disconnects_when_subscribe_fails",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_start_disconnects_when_bind_fails() {
        super::timing::asynchronous(
            "aligned::status_stream::test_start_disconnects_when_bind_fails",
            super::stream_cases::run("status_stream", "test_start_disconnects_when_bind_fails"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_run_forever_raises_when_connection_lost() {
        super::timing::asynchronous(
            "aligned::status_stream::test_run_forever_raises_when_connection_lost",
            super::stream_cases::run(
                "status_stream",
                "test_run_forever_raises_when_connection_lost",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_run_forever_reraises_transport_error_unwrapped() {
        super::timing::asynchronous(
            "aligned::status_stream::test_run_forever_reraises_transport_error_unwrapped",
            super::stream_cases::run(
                "status_stream",
                "test_run_forever_reraises_transport_error_unwrapped",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_restart_after_connection_loss_clears_the_error() {
        super::timing::asynchronous(
            "aligned::status_stream::test_restart_after_connection_loss_clears_the_error",
            super::stream_cases::run(
                "status_stream",
                "test_restart_after_connection_loss_clears_the_error",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_context_manager_starts_and_stops() {
        super::timing::asynchronous(
            "aligned::status_stream::test_context_manager_starts_and_stops",
            super::stream_cases::run("status_stream", "test_context_manager_starts_and_stops"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_handle_message_ignores_non_property_topic() {
        super::timing::asynchronous(
            "aligned::status_stream::test_handle_message_ignores_non_property_topic",
            super::stream_cases::run(
                "status_stream",
                "test_handle_message_ignores_non_property_topic",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_handle_message_drops_unparsable_payload() {
        super::timing::asynchronous(
            "aligned::status_stream::test_handle_message_drops_unparsable_payload",
            super::stream_cases::run(
                "status_stream",
                "test_handle_message_drops_unparsable_payload",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_handle_message_skips_empty_changes() {
        super::timing::asynchronous(
            "aligned::status_stream::test_handle_message_skips_empty_changes",
            super::stream_cases::run("status_stream", "test_handle_message_skips_empty_changes"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_handle_message_dispatches_and_fires_on_change() {
        super::timing::asynchronous(
            "aligned::status_stream::test_handle_message_dispatches_and_fires_on_change",
            super::stream_cases::run(
                "status_stream",
                "test_handle_message_dispatches_and_fires_on_change",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_handle_message_swallows_handler_exception() {
        super::timing::asynchronous(
            "aligned::status_stream::test_handle_message_swallows_handler_exception",
            super::stream_cases::run(
                "status_stream",
                "test_handle_message_swallows_handler_exception",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_fire_typed_bool_events() {
        super::timing::asynchronous(
            "aligned::status_stream::test_fire_typed_bool_events",
            super::stream_cases::run("status_stream", "test_fire_typed_bool_events"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_fire_typed_int_str_and_state_events() {
        super::timing::asynchronous(
            "aligned::status_stream::test_fire_typed_int_str_and_state_events",
            super::stream_cases::run("status_stream", "test_fire_typed_int_str_and_state_events"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_fire_typed_unknown_goes_to_on_unknown() {
        super::timing::asynchronous(
            "aligned::status_stream::test_fire_typed_unknown_goes_to_on_unknown",
            super::stream_cases::run(
                "status_stream",
                "test_fire_typed_unknown_goes_to_on_unknown",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_flatten_skips_non_mapping_entries() {
        super::timing::asynchronous(
            "aligned::status_stream::test_flatten_skips_non_mapping_entries",
            super::stream_cases::run("status_stream", "test_flatten_skips_non_mapping_entries"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_fire_typed_without_handlers_is_noop() {
        super::timing::asynchronous(
            "aligned::status_stream::test_fire_typed_without_handlers_is_noop",
            super::stream_cases::run("status_stream", "test_fire_typed_without_handlers_is_noop"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_reconnects_after_connection_loss() {
        super::timing::asynchronous(
            "aligned::status_stream::test_reconnects_after_connection_loss",
            super::stream_cases::run("status_stream", "test_reconnects_after_connection_loss"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_reconnect_retries_until_the_broker_comes_back() {
        super::timing::asynchronous(
            "aligned::status_stream::test_reconnect_retries_until_the_broker_comes_back",
            super::stream_cases::run(
                "status_stream",
                "test_reconnect_retries_until_the_broker_comes_back",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_reconnect_gives_up_and_notifies_the_consumer() {
        super::timing::asynchronous(
            "aligned::status_stream::test_reconnect_gives_up_and_notifies_the_consumer",
            super::stream_cases::run(
                "status_stream",
                "test_reconnect_gives_up_and_notifies_the_consumer",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_second_drop_does_not_stack_reconnect_tasks() {
        super::timing::asynchronous(
            "aligned::status_stream::test_second_drop_does_not_stack_reconnect_tasks",
            super::stream_cases::run(
                "status_stream",
                "test_second_drop_does_not_stack_reconnect_tasks",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_stop_cancels_a_pending_reconnect() {
        super::timing::asynchronous(
            "aligned::status_stream::test_stop_cancels_a_pending_reconnect",
            super::stream_cases::run("status_stream", "test_stop_cancels_a_pending_reconnect"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_drop_while_stopping_is_not_retried() {
        super::timing::asynchronous(
            "aligned::status_stream::test_drop_while_stopping_is_not_retried",
            super::stream_cases::run("status_stream", "test_drop_while_stopping_is_not_retried"),
        )
        .await;
    }
    #[tokio::test]
    async fn test_reconnect_policy_rejects_impossible_schedules() {
        super::timing::asynchronous(
            "aligned::status_stream::test_reconnect_policy_rejects_impossible_schedules",
            super::stream_cases::run(
                "status_stream",
                "test_reconnect_policy_rejects_impossible_schedules",
            ),
        )
        .await;
    }
    #[tokio::test]
    async fn test_reconnect_policy_backs_off_up_to_the_ceiling() {
        super::timing::asynchronous(
            "aligned::status_stream::test_reconnect_policy_backs_off_up_to_the_ceiling",
            super::stream_cases::run(
                "status_stream",
                "test_reconnect_policy_backs_off_up_to_the_ceiling",
            ),
        )
        .await;
    }
}
mod status_update {
    #[test]
    fn test_known_property_mapping_is_snake_case() {
        super::timing::sync(
            "aligned::status_update::test_known_property_mapping_is_snake_case",
            || super::pure::run("status_update", "test_known_property_mapping_is_snake_case"),
        );
    }
    #[test]
    fn test_room_of_bin_maps_to_bucket_full() {
        super::timing::sync(
            "aligned::status_update::test_room_of_bin_maps_to_bucket_full",
            || super::pure::run("status_update", "test_room_of_bin_maps_to_bucket_full"),
        );
    }
    #[test]
    fn test_bucket_status_maps_to_operating_state() {
        super::timing::sync(
            "aligned::status_update::test_bucket_status_maps_to_operating_state",
            || {
                super::pure::run(
                    "status_update",
                    "test_bucket_status_maps_to_operating_state",
                )
            },
        );
    }
    #[test]
    fn test_unknown_property_passes_through_under_camelcase() {
        super::timing::sync(
            "aligned::status_update::test_unknown_property_passes_through_under_camelcase",
            || {
                super::pure::run(
                    "status_update",
                    "test_unknown_property_passes_through_under_camelcase",
                )
            },
        );
    }
    #[test]
    fn test_missing_device_or_items_drops_to_none() {
        super::timing::sync(
            "aligned::status_update::test_missing_device_or_items_drops_to_none",
            || {
                super::pure::run(
                    "status_update",
                    "test_missing_device_or_items_drops_to_none",
                )
            },
        );
    }
}
