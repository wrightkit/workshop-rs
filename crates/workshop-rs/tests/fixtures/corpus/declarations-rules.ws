variables {
    global:
        0: score
    player:
        0: hasStarted
}

subroutines {
    0: showStatus
}

rule ("Subroutine showStatus") {
    event {
        Subroutine;
        showStatus;
    }
    actions {
        Create HUD Text(All Players(All Teams), Custom String("Score: {0}", Global.score), Custom String("                                                                                                                             {0}", Custom String("                                             ")), Null, Left, -9999, Color(Orange), Null, Null, Visible To and String, Default Visibility);
    }
}

rule ("player starts") {
    event {
        Ongoing - Each Player;
        All;
        All;
    }
    conditions {
        Has Spawned(Event Player) == True;
    }
    actions {
        Set Player Variable(Event Player, hasStarted, True);
        Call Subroutine(showStatus);
    }
}
