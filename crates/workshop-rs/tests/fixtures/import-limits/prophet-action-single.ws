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
        Create In-World Text(If-Then-Else(Compare(If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null), !=, Null), Filtered Array(All Players(Team 2), And(And(Compare(Current Array Element, !=, If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null)), Compare((Current Array Element).eventType, ==, 2)), Compare((Current Array Element).eventId, ==, 23))), Empty Array), If-Then-Else(Compare(If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null), !=, Null), Custom String("→ {0}", If-Then-Else(Compare((If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null)).eventNextIndex, !=, Null), Value In Array(Global.eventName, (If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null)).eventNextIndex), Custom String("未知"))), Char In String(Empty Array, False)), If-Then-Else(Compare(If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null), !=, Null), Add(Eye Position(If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null)), Multiply(0.3, Up)), Multiply(-1000, Up)), 0.7, Do Not Clip, Visible To Position String and Color, If-Then-Else(And(Compare(If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null), !=, Null), Compare((If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null)).playerTitleAndColor, !=, Null)), Custom Color(X Component Of((If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null)).rgb_vect), Y Component Of((If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null)).rgb_vect), Z Component Of((If-Then-Else(Compare(Count Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), >, Null), First Of(Filtered Array(All Players(Team 2), Compare(Slot Of(Current Array Element), ==, Null))), Null)).rgb_vect), 255), Custom Color(186, 120, 255, 255)), Default Visibility);
        Set Global Variable(prophetSlotTextsCreated, True);
    }
    }
