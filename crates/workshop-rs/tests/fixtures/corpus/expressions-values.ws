variables {
    global:
        0: result
        1: points
        2: location
}

rule ("Initialize global variables") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(points, Array(1, 2, 3));
        Set Global Variable(location, Vector(1, 2, 3));
    }
}

rule ("expressions and values") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(result, Add(Count Of(Global.points), 6));
        Modify Global Variable(points, Append To Array, Global.result);
        Create HUD Text(All Players(All Teams), Custom String("points: {0}", Global.points), Custom String("                                                                                                                             {0}", Custom String("                                             ")), Null, Left, -9999, Color(Orange), Null, Null, Visible To and String, Default Visibility);
        Create HUD Text(All Players(All Teams), Null, Custom String("Iοƈạṭіοṇ = {0}                                                                                                               {1}", Mapped Array(Mapped Array(Array(Mapped Array(Global.location, If-Then-Else(Or(Compare(Count Of(Current Array Element), ==, 1), And(Compare(Current Array Element, ==, Empty Array), Compare(Current Array Element, !=, Null))), Custom String("[{0}]", Current Array Element), If-Then-Else(Count Of(Current Array Element), Custom String("[{0}, …+{1}]", Current Array Element, Subtract(Count Of(Current Array Element), 1)), Current Array Element)))), Append To Array(Append To Array(Or(Count Of(Current Array Element), And(Compare(Current Array Element, ==, Empty Array), Compare(Current Array Element, !=, Null))), If-Then-Else(And(Not(Count Of(Current Array Element)), Compare(Current Array Element, !=, Empty Array)), 3, Multiply(Count Of(Current Array Element), 3))), Current Array Element)), If-Then-Else(First Of(Current Array Element), Custom String("[{0}{1}]", String Replace(Custom String("{0}, {1}, {2}", Value In Array(Current Array Element, 2), Value In Array(Current Array Element, 3), Custom String("{0}, {1}, {2}", Value In Array(Current Array Element, 4), Value In Array(Current Array Element, 5), Custom String("{0}, {1}, …", Value In Array(Current Array Element, 6), Value In Array(Current Array Element, 7)))), String Slice(Custom String("0, 0, 0, 0, 0, 0, …"), Add(-2, Value In Array(Current Array Element, 1)), Subtract(22, Value In Array(Current Array Element, 1))), Empty Array), If-Then-Else(Compare(Value In Array(Current Array Element, 1), >, 18), Custom String("+{0}", Subtract(Divide(Value In Array(Current Array Element, 1), 3), 6)), Empty Array)), String Split(Value In Array(Current Array Element, 2), Empty Array))), Custom String("                                                           ")), Null, Left, -9999, Null, Color(White), Null, Visible To Sort Order String and Color, Default Visibility);
    }
}
