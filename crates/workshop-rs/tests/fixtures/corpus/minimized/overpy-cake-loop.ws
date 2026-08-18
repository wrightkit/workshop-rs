variables {
    global:
        0: candlePos
        1: i2
}

rule ("cake-minimized-loop") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(candlePos, Empty Array);
        For Global Variable(i2, 0, 1, 1);
            Modify Global Variable(candlePos, Append To Array, Vector(Random Real(-1, 1), 0, Random Real(-1, 1)));
        End;
    }
}
