use arcrtc_roadmap_tests::assert_impl_file_contains;

#[test]
fn sfu_server_source_has_udp_send_path_shape() {
    assert_impl_file_contains(
        "entrypoints/sfu-server/src/main.rs",
        &[
            "UdpSocket::bind",
            "recv_from",
            "ingest_udp_datagram",
            "drain_outbound",
            "send_to",
            "outcome=ok transmits=",
        ],
    );
}
