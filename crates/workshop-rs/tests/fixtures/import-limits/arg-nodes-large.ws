variables {
    global:
        0: probe
}

rule ("probe: wide value tree") {
    event {
        Ongoing - Global;
    }
    conditions {
        True;
    }
    actions {
        Set Global Variable(probe, And(And(And(And(And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))), And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))))), And(And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))), And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))))), And(And(And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))), And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))))), And(And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))), And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))))))), And(And(And(And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))), And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))))), And(And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))), And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))))), And(And(And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))), And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))))), And(And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))), And(And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True))), And(And(And(True, True), And(True, True)), And(And(True, True), And(True, True)))))))));
    }
}
