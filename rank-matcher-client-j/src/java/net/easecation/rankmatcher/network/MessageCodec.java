package net.easecation.rankmatcher.network;

import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.MessageToMessageCodec;
import io.netty.handler.codec.http.websocketx.TextWebSocketFrame;
import net.easecation.rankmatcher.api.CharReader;
import net.easecation.rankmatcher.api.Message;

import java.util.List;

public class MessageCodec extends MessageToMessageCodec<TextWebSocketFrame, Message> {

    @Override
    protected void decode(ChannelHandlerContext channelHandlerContext, TextWebSocketFrame frame, List<Object> list) {
        CharReader reader = new CharReader(frame.text());
        Message message = reader.readPacket();
        message.decode(reader);
        list.add(message);
    }

    @Override
    protected void encode(ChannelHandlerContext channelHandlerContext, Message message, List<Object> list) {
        list.add(new TextWebSocketFrame(message.toString()));
    }

}
