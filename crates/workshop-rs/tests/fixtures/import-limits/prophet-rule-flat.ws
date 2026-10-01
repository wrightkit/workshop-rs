variables {
    global:
        0: prophetSlotTextsCreated
        1: eventName
    player:
        0: eventNextIndex
        1: eventType
        2: eventId
        3: playerTitleAndColor
        4: rgb_vect
}

rule ("[Event/先知] Create per-slot prophet next-event texts") {
    event {
        Ongoing - Global;
    }
    conditions {
        Global.prophetSlotTextsCreated == False;
        Count Of(All Players(Team 2)) > Null;
    }
    actions {
        Create In-World Text(All Players(Team 2), Null, Event Player, 1, Do Not Clip, Visible To Position String and Color, Color(White), Default Visibility);
        Create In-World Text(All Players(Team 2), Null, Event Player, 1, Do Not Clip, Visible To Position String and Color, Color(White), Default Visibility);
        Create In-World Text(All Players(Team 2), Null, Event Player, 1, Do Not Clip, Visible To Position String and Color, Color(White), Default Visibility);
        Create In-World Text(All Players(Team 2), Null, Event Player, 1, Do Not Clip, Visible To Position String and Color, Color(White), Default Visibility);
        Create In-World Text(All Players(Team 2), Null, Event Player, 1, Do Not Clip, Visible To Position String and Color, Color(White), Default Visibility);
        Create In-World Text(All Players(Team 2), Null, Event Player, 1, Do Not Clip, Visible To Position String and Color, Color(White), Default Visibility);
        Create In-World Text(All Players(Team 2), Null, Event Player, 1, Do Not Clip, Visible To Position String and Color, Color(White), Default Visibility);
        Create In-World Text(All Players(Team 2), Null, Event Player, 1, Do Not Clip, Visible To Position String and Color, Color(White), Default Visibility);
        Set Global Variable(prophetSlotTextsCreated, True);
    }
}
