variables {
    global:
        0: g
}

rule ("r") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Set Global Variable(g, Custom Color(255, 128, False, 255));
        Set Damage Dealt(Event Player, False);
        Set Global Variable(g, Direction From Angles(False, 90));
        Set Global Variable(g, Custom Color(255, 128, True, 255));
        Set Damage Dealt(Event Player, True);
        Set Global Variable(g, Direction From Angles(True, 90));
        Set Global Variable(g, Custom Color(False, 128, 255, 255));
        Set Global Variable(g, Custom Color(True, 128, 255, 255));
        Set Global Variable(g, Custom Color(255, False, 128, 255));
        Set Global Variable(g, Custom Color(255, True, 128, 255));
        Set Global Variable(g, Custom Color(255, 128, 64, False));
        Set Global Variable(g, Custom Color(255, 128, 64, True));
        Set Global Variable(g, Direction From Angles(45, False));
        Set Global Variable(g, Direction From Angles(45, True));
        Set Damage Received(Event Player, False);
        Set Damage Received(Event Player, True);
        Set Knockback Received(Event Player, False);
        Set Knockback Received(Event Player, True);
        Start Damage Modification(Event Player, Event Player, False, None);
        Start Damage Modification(Event Player, Event Player, True, None);
        Start Throttle In Direction(Event Player, Up, False, To World, Replace existing throttle, None);
        Start Throttle In Direction(Event Player, Up, True, To World, Replace existing throttle, None);
        Set Global Variable(g, Is In View Angle(Event Player, Vector(1, 2, 3), False));
        Set Global Variable(g, Is In View Angle(Event Player, Vector(1, 2, 3), True));
        Play Effect(All Players(All Teams), Bad Explosion, Color(Red), Event Player, False);
        Play Effect(All Players(All Teams), Bad Explosion, Color(Red), Event Player, True);
    }
}
