package net.easecation.rankmatcher.network;

import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.MessageToMessageCodec;
import io.netty.handler.codec.http.websocketx.TextWebSocketFrame;
import net.easecation.rankmatcher.api.Message;

import java.util.List;
import java.util.function.Supplier;

public class MessageCodec extends MessageToMessageCodec<TextWebSocketFrame, Message> {

    @Override
    protected void decode(ChannelHandlerContext channelHandlerContext, TextWebSocketFrame frame, List<Object> list) {
        String[] split = frame.text().split(",");
        Supplier<Message> messageSupplier = Message.MESSAGE_SUPPLIERS.get(split[1]);
        if (messageSupplier != null) {
            Message message = messageSupplier.get();
            message.decode(split);
            list.add(message);
        } else {
            throw new IllegalArgumentException("Unknown message type: " + split[1]);
        }
    }

    @Override
    protected void encode(ChannelHandlerContext channelHandlerContext, Message message, List<Object> list) {
        list.add(new TextWebSocketFrame(message.toString()));
    }

}
