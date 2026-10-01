variables {
    global:
        0: probe
}

rule ("control: minimal") {
    event {
        Ongoing - Global;
    }
    conditions {
        True;
    }
    actions {
        Set Global Variable(probe, 0);
    }
}
