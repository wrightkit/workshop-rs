variables {
    global:
        0: target
}

rule ("move speed") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    conditions {
        Is Alive(Event Player) == True;
    }
    actions {
        Set Move Speed(Event Player, 100);
    }
}

rule ("combat teleport") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Set Max Health(Event Player, 200);
        Set Player Health(Event Player, 200);
        Teleport(Event Player, Position Of(Event Player));
    }
}

rule ("utility adjustments") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Set Aim Speed(Event Player, 150);
        Set Gravity(Event Player, 100);
        Set Damage Dealt(Event Player, 50);
        Set Damage Received(Event Player, 50);
    }
}

rule ("conditional support") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    conditions {
        Health(Event Player) < 50;
        Has Spawned(Event Player) == True;
    }
    actions {
        Set Move Speed(Event Player, 75);
        Set Ultimate Charge(Event Player, 100);
    }
}

rule ("player receiver") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    actions {
        Set Global Variable(target, Event Player);
        Set Move Speed(Global.target, 50);
    }
}
