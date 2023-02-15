package net.easecation.rankmatcher.api;

import net.easecation.rankmatcher.api.message.*;

import java.util.HashMap;
import java.util.Map;
import java.util.function.Supplier;

/*
* 封装一条 消息数据
* */
public interface Message {

    String PROTOCOL_VERSION = "1";

    /**
     * 用于解码的消息类型（所以服务端不发的，可以不注册）
     */
    Map<MessageType, Supplier<Message>> MESSAGE_SUPPLIERS = new HashMap<MessageType, Supplier<Message>>(){{
        put(MessageType.ADD_ARENA, AddArenaMessage::new);
        put(MessageType.REMOVE_ARENA, RemoveArenaMessage::new);
        put(MessageType.ADD_PLAYER, AddPlayerMessage::new);
        put(MessageType.REMOVE_PLAYER, RemovePlayerMessage::new);
        put(MessageType.GET_OR_SUBSCRIBE_STATE, GetOrSubscribeStateMessage::new);
        put(MessageType.CONNECTION_STATE, ConnectionStateMessage::new);
        put(MessageType.MATCH_SUCCESS, MatchSuccessMessage::new);
        put(MessageType.MATCH_FAILURE, MatchFailureMessage::new);
        put(MessageType.FORMAT_ERROR, FormatErrorMessage::new);
    }};

    MessageType getMessageType();

    default void decode(String[] data) {}

    String toString();

    static String writeString(String string) {
        string = string.replace(",", "\\_");
        return string.length() + "," + string;
    }

}
