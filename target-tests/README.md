# Target Tests

*Hardware in the loop (HIL) test suite for the
[pa-spl](https://github.com/EngJay/pa-spl) driver.*

## Running the Tests

The HIL tests are configured to use a 0x32 or 0x33 version of the PCB Artists
I2C SPL Module connected to the I2C1 interface of a STM32F3 Discovery
development board.

Set up the hardware:

- connect the SCL, SDA, 3.3V, and ground pins of the sensor to pins PB6, PB7,
  a 3.3V, and a ground pin on the dev board, respectively.
- connect the dev board to the machine running the tests via its ST-Link USB
  connection.  

Then, run the tests with cargo. The received response is expected to be similar
to the following:

```cli
cargo test

... some build messages and warnings ...

    Finished `test` profile [optimized + debuginfo] target(s) in 2.18s
     Running tests/pa-spl.rs (target/thumbv7em-none-eabihf/debug/deps/pa_spl-a7b0d3b10a1b3b25)
      Erasing ✔ [00:00:00] [#######################]       0 B/      0 B @       0 B/s (eta 0s )
  Programming ✔ [00:00:01] [#######################] 14.00 KiB/14.00 KiB @ 12.82 KiB/s (eta 0s )    Finished in 1.1188269s
<lvl> (1/12) running `confirm_firmware_version`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:94  
<lvl> (2/12) running `confirm_device_id`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:102 
<lvl> (3/12) running `confirm_get_avg_time`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:110 
<lvl> (4/12) running `confirm_get_control_register`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:117 
<lvl> (5/12) running `confirm_get_max_decibel`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:124 
<lvl> (6/12) running `confirm_get_min_decibel`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:132 
<lvl> (7/12) running `confirm_read_latest_decibel`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:140 
<lvl> (8/12) running `confirm_reset`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:148 
<lvl> (9/12) running `confirm_rw_scratch`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:163 
<lvl> (10/12) running `confirm_set_avg_time`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:173 
<lvl> (11/12) running `confirm_set_control_register`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:184 
<lvl> (12/12) running `sanity_check`...
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:199 
<lvl> all tests passed!
└─ pa_spl::tests::__defmt_test_entry @ tests/pa-spl.rs:40  

```
