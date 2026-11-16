variables {
    global:
        0: probe
        1: t
}

rule ("slot acceptance: combined") {
    event {
        Ongoing - Global;
    }
    actions {
        Wait(1, Ignore Condition);
        Set Global Variable(t, 7);
        Log To Inspector(Custom String("SA1 control={0}", String Slice(Custom String("abcdef"), 1, 2), Null, Null));
        Log To Inspector(Custom String("SA2 number_literal={0}", String Slice(100, 1, 2), Null, Null));
        Log To Inspector(Custom String("SA3 number_expression={0}", String Slice(Add(Modulo(Global.t, 60), 100), 1, 2), Null, Null));
        Log To Inspector(Custom String("SA4 vector={0}", String Slice(Vector(1, 2, 3), 1, 2), Null, Null));
        Log To Inspector(Custom String("SA5 player={0}", String Slice(Host Player, 1, 2), Null, Null));
        Log To Inspector(Custom String("SA6 square_root_player={0}", Square Root(Host Player), Null, Null));
        Log To Inspector(Custom String("SA done", Null, Null, Null));
    }
}
