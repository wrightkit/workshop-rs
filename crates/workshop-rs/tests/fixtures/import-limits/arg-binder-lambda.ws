variables {
    global:
        0: probe
}

rule ("probe: binder in per-element argument") {
    event {
        Ongoing - Global;
    }
    conditions {
        True;
    }
    actions {
        Set Global Variable(probe, Count Of(Filtered Array(All Players(Team 1), Is True For All(Array(Is Alive(Current Array Element), Compare(Current Array Element, !=, Null)), Current Array Element))));
    }
}
