package net.easecation.rankmatcher.api;

import io.netty.channel.Channel;
import io.netty.util.concurrent.Future;
import lombok.extern.log4j.Log4j2;
import net.easecation.rankmatcher.RankMatcherClient;

import java.util.Timer;

@Log4j2
public class MessageSender {

    private final RankMatcherClient client;
    private final Channel channel;
    private final Timer timer;

    public Channel getChannel() {
        return channel;
    }

    public MessageSender(RankMatcherClient client, Channel channel) {
        this.client = client;
        this.channel = channel;
        this.timer = new Timer();
    }

    public void stopTimer() {
        timer.cancel();
    }

    /**
     * 发送消息 同步方式 不使用Result处理返回值
     */
    public boolean sendSyncMessage(Message message) {
        try {
            return getChannel().writeAndFlush(message).sync().isSuccess();
        } catch (InterruptedException e) {
            log.catching(e);
            return false;
        }
    }

    /**
     * 发送消息 同步方式 使用Result处理返回值
     */
    public void sendSyncMessage(Message message, Result result) {
        try {
            Future<?> future = getChannel().writeAndFlush(message).sync();
            if (result != null) result.handle(future);
        } catch (InterruptedException e) {
            log.catching(e);
        }
    }

    /**
     * 异步发送
     */
    public void sendAsyncMessage(Message message, Result result) {
        this.channel.writeAndFlush(message).addListener(result::handle);
    }

    @FunctionalInterface
    public interface Result {
        void handle(Future<?> future) throws InterruptedException;
    }
}
