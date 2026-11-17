variables {
    global:
        0: probe
        1: t
}

rule ("slot acceptance: round 2") {
    event {
        Ongoing - Global;
    }
    actions {
        Wait(1, Ignore Condition);
        Set Global Variable(t, 7);
        Log To Inspector(Custom String("SB1 control={0}", String Slice(Custom String("abcdef"), 1, 2), Null, Null));
        Log To Inspector(Custom String("SB2 abs_call={0}", String Slice(Absolute Value(Global.t), 1, 2), Null, Null));
        Log To Inspector(Custom String("SB3 vector_dynamic={0}", String Slice(Vector(Global.t, 0, 0), 1, 2), Null, Null));
        Log To Inspector(Custom String("SB4 add_literals={0}", String Slice(Add(100, 0), 1, 2), Null, Null));
        Log To Inspector(Custom String("SB5 variable={0}", String Slice(Global.t, 1, 2), Null, Null));
        Log To Inspector(Custom String("SB6 sqrt_vector_const={0}", Square Root(Vector(1, 2, 3)), Null, Null));
        Log To Inspector(Custom String("SB7 sqrt_string={0}", Square Root(Custom String("a")), Null, Null));
        Log To Inspector(Custom String("SB done", Null, Null, Null));
    }
}
