variables {
    global:
        0: g
        1: v
}

rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Set Global Variable(v, Subtract(Event Player, Up));
        Set Global Variable(v, Subtract(Up, Event Player));
        Set Global Variable(v, Add(Event Player, Up));
        Set Global Variable(v, Add(Up, Event Player));
        Set Global Variable(v, Multiply(Up, Event Player));
        Set Global Variable(v, Divide(Event Player, 2));
        Set Global Variable(v, Divide(Up, Event Player));
        Set Global Variable(v, Ray Cast Hit Position(Event Player, Add(Event Player, Vector(0, -100, 0)), Null, Null, False));
        Set Global Variable(v, Nearest Walkable Position(Event Player));
        Teleport(Event Player, Subtract(Event Player, Up));
        Set Global Variable(g, Is In View Angle(Event Player, Event Player, 10));
    }
}
