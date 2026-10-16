variables {
    global:
        0: probe
}

rule ("probe: binder in source argument") {
    event {
        Ongoing - Global;
    }
    conditions {
        True;
    }
    actions {
        Set Global Variable(probe, Count Of(Filtered Array(Filtered Array(All Players(Team 1), Is Alive(Current Array Element)), Compare(Current Array Element, !=, Null))));
    }
}
