variables {
    global:
        0: v
}

rule ("x") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(v, -Infinity);
        Set Global Variable(v, NaN);
        Set Global Variable(v, Infinity);
    }
}
