variables {
    global:
        0: probe
}

rule ("probe: deep argument") {
    event {
        Ongoing - Global;
    }
    conditions {
        True;
    }
    actions {
        Set Global Variable(probe, If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), If-Then-Else(Compare(1, <, 2), 0, 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0), 0));
    }
}
