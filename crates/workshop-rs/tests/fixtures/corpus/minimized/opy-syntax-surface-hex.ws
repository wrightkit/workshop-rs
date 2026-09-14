variables {
    global:
        0: value
}

rule ("issue 28 syntax") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(value, Array(16, 0X20));
    }
}
