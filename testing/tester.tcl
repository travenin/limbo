set sqlite_exec [expr {[info exists env(SQLITE_EXEC)] ? $env(SQLITE_EXEC) : "sqlite3"}]

proc evaluate_sql {sqlite_exec sql} {
    set command [list $sqlite_exec testing/testing.db $sql]
    set output [exec {*}$command]
    return $output
}

proc run_test {sqlite_exec sql expected_output} {
    set actual_output [evaluate_sql $sqlite_exec $sql]
    if {$actual_output ne $expected_output} {
        puts "Test FAILED: '$sql'"
        puts "returned '$actual_output'"
        puts "expected '$expected_output'"
        exit 1
    }
}

proc do_execsql_test {test_name sql_statements expected_outputs} {
    puts "Running test: $test_name"
    set combined_sql [string trim $sql_statements]
    set combined_expected_output [join $expected_outputs "\n"]
    run_test $::sqlite_exec $combined_sql $combined_expected_output
}

proc within_tolerance {actual expected tolerance} {
    expr {abs($actual - $expected) <= $tolerance}
}

proc do_execsql_test_tolerance {test_name sql_statements expected_outputs tolerance} {
    puts "Running test: $test_name"
    set combined_sql [string trim $sql_statements]
    set actual_output [evaluate_sql $::sqlite_exec $combined_sql]
    set actual_values [split $actual_output "\n"]
    set expected_values [split $expected_outputs "\n"]

    if {[llength $actual_values] != [llength $expected_values]} {
        puts "Test FAILED: '$sql_statements'"
        puts "returned '$actual_output'"
        puts "expected '$expected_outputs'"
        exit 1
    }

    for {set i 0} {$i < [llength $actual_values]} {incr i} {
        set actual [lindex $actual_values $i]
        set expected [lindex $expected_values $i]

        if {![within_tolerance $actual $expected $tolerance]} {
            set lower_bound [expr {$expected - $tolerance}]
            set upper_bound [expr {$expected + $tolerance}]
            puts "Test FAILED: '$sql_statements'"
            puts "returned '$actual'"
            puts "expected a value within the range \[$lower_bound, $upper_bound\]"
            exit 1
        }
    }
}
