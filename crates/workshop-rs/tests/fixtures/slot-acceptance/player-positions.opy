globalvar g
globalvar v

rule "r":
    @Event eachPlayer
    v = eventPlayer - Vector.UP
    v = Vector.UP - eventPlayer
    v = eventPlayer + Vector.UP
    v = Vector.UP + eventPlayer
    v = Vector.UP * eventPlayer
    v = eventPlayer / 2
    v = Vector.UP / eventPlayer
    v = raycast(eventPlayer, eventPlayer + Vector.DOWN * 100, null, null, false).getHitPosition()
    v = nearestWalkablePosition(eventPlayer)
    eventPlayer.teleport(eventPlayer - Vector.UP)
    g = eventPlayer.isInViewAngle(eventPlayer, 10)
