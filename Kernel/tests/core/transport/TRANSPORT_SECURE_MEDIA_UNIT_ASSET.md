# Core Transport And Secure Media Unit Asset

Task: T2.5

Correlation field: required.
Evidence class: test.
Close-not-claimed: transport unit evidence does not prove concrete DTLS/SRTP runtime.
Rerun condition: rerun when transport, SDP/ICE, ICE lifecycle, or secure media Canonical changes.

This unit-test asset is owned by core and covers Sans-IO transport contract, SDP/ICE references, ICE candidate lifecycle, connectivity lifecycle, and secure media session lifecycle.

forbidden driver ownership: concrete I/O, socket, browser, native, and str0m behavior remain drivers.
