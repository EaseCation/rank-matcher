package net.easecation.rankmatcher.api;

public enum MessageType {

    ADD_ARENA(1),
    REMOVE_ARENA(2),
    ADD_PLAYER(3),
    REMOVE_PLAYER(4),
    GET_OR_SUBSCRIBE_STATE(5),
    CONNECTION_STATE(6),
    MATCH_SUCCESS(7),
    MATCH_FAILURE(8),
    FORMAT_ERROR(9);

    private final int id;

    MessageType(int id) {
        this.id = id;
    }

    public int getTypeId() {
        return id;
    }
}
