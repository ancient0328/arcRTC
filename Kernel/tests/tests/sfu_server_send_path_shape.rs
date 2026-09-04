use arcrtc_kernel_test_suite::assert_impl_file_contains;

#[test]
fn sfu_server_source_has_udp_send_path_shape() {
    assert_impl_file_contains(
        "entrypoints/composition-root/src/lib.rs",
        &[
            "UdpSocket::bind",
            "recv_from",
            "decode_sfu_media_datagram",
            "apply_resident_sfu_input",
            "resident_sfu_response",
            "send_to",
            "response.datagram",
            "outcome=ok {detail_code} transmits=",
        ],
    );
    assert_impl_file_contains(
        "core/sfu/src/lib_parts/part_004.rs",
        &[
            "ResidentSfuState",
            "ResidentSfuInputClass",
            "decide_secure_media_session",
            "apply_sfu_forwarding",
            "ResidentSfuOutcome::Accepted",
        ],
    );
    assert_impl_file_contains(
        "entrypoints/sfu-server/src/main.rs",
        &["serve_resident_loop(config)"],
    );
}
