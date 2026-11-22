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
        Create Effect(All Players(All Teams), Sphere, Custom Color(255, 112, 122, 255), False, 1, Visible To Position Radius and Color);
        Create Effect(All Players(All Teams), Sphere, Custom Color(255, 112, 122, 255), True, 1, Visible To Position Radius and Color);
        Play Effect(All Players(All Teams), Bad Explosion, Custom Color(255, 112, 122, 255), False, 1);
        Play Effect(All Players(All Teams), Bad Explosion, Custom Color(255, 112, 122, 255), True, 1);
        Create Icon(All Players(All Teams), False, Flag, Visible To and Position, Custom Color(255, 112, 122, 255), True);
        Create Icon(All Players(All Teams), True, Flag, Visible To and Position, Custom Color(255, 112, 122, 255), True);
        Create In-World Text(All Players(All Teams), Custom String("a"), False, 1, Do Not Clip, Visible To Position and String, Custom Color(255, 112, 122, 255), Default Visibility);
        Create In-World Text(All Players(All Teams), Custom String("a"), True, 1, Do Not Clip, Visible To Position and String, Custom Color(255, 112, 122, 255), Default Visibility);
        Create Progress Bar In-World Text(All Players(All Teams), 1, Custom String("a"), False, 1, Do Not Clip, Custom Color(255, 112, 122, 255), Custom Color(255, 112, 122, 255), Visible To Position and Color, Default Visibility);
        Create Progress Bar In-World Text(All Players(All Teams), 1, Custom String("a"), True, 1, Do Not Clip, Custom Color(255, 112, 122, 255), Custom Color(255, 112, 122, 255), Visible To Position and Color, Default Visibility);
        Create Projectile Effect(All Players(All Teams), Roadhog Scrap, Event Player, False, Up, 1, Visible To Position Direction and Size);
        Create Projectile Effect(All Players(All Teams), Roadhog Scrap, Event Player, True, Up, 1, Visible To Position Direction and Size);
        Set Global Variable(g, Is In Line of Sight(False, Event Player, Barriers Do Not Block LOS));
        Set Global Variable(g, Is In Line of Sight(True, Event Player, Barriers Do Not Block LOS));
        Set Global Variable(g, Is In Line of Sight(Event Player, False, Barriers Do Not Block LOS));
        Set Global Variable(g, Is In Line of Sight(Event Player, True, Barriers Do Not Block LOS));
        Set Global Variable(g, Distance Between(False, Event Player));
        Set Global Variable(g, Distance Between(True, Event Player));
        Set Global Variable(g, Distance Between(Event Player, False));
        Set Global Variable(g, Distance Between(Event Player, True));
        Set Global Variable(g, Direction Towards(False, Event Player));
        Set Global Variable(g, Direction Towards(True, Event Player));
        Set Global Variable(g, Direction Towards(Event Player, False));
        Set Global Variable(g, Direction Towards(Event Player, True));
        Set Global Variable(g, Players Within Radius(False, 1, All Teams, Off));
        Set Global Variable(g, Players Within Radius(True, 1, All Teams, Off));
        Set Global Variable(g, Closest Player To(False, All Teams));
        Set Global Variable(g, Closest Player To(True, All Teams));
        Set Global Variable(g, Horizontal Angle Towards(Event Player, False));
        Set Global Variable(g, Horizontal Angle Towards(Event Player, True));
    }
}
