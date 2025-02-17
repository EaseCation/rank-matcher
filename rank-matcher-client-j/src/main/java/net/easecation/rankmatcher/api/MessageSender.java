package net.easecation.rankmatcher.api;

import io.netty.channel.Channel;
import io.netty.util.concurrent.Future;
import lombok.extern.log4j.Log4j2;
import net.easecation.rankmatcher.RankMatcherClient;

import java.util.Queue;
import java.util.Timer;
import java.util.concurrent.LinkedBlockingQueue;

@Log4j2
public class MessageSender {

    private final RankMatcherClient client;
    private Channel channel;
    private final Timer timer;

    private boolean isHandshake = false;

    private final Queue<Message> messagesBeforeHandshake = new LinkedBlockingQueue<>();

    public Channel getChannel() {
        return channel;
    }

    public MessageSender(RankMatcherClient client) {
        this.client = client;
        this.timer = new Timer();
    }

    public void stopTimer() {
        timer.cancel();
    }

    public MessageSender onHandshakeSuc(Channel channel) {
        if (!isHandshake) {
            this.channel = channel;
            isHandshake = true;  // 先设置上去，防止继续往里面添加
            while (!messagesBeforeHandshake.isEmpty()) {
                Message message = messagesBeforeHandshake.poll();
                if (message != null) {
                    sendSyncMessage(message);
                }
            }
        }
        return this;
    }

    /**
     * 发送消息 同步方式 不使用Result处理返回值
     */
    public boolean sendSyncMessage(Message message) {
        if (!this.isHandshake) {
            this.messagesBeforeHandshake.add(message);
            return true;
        }
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
        if (!this.isHandshake) {
            this.messagesBeforeHandshake.add(message);
            return;
        }
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
        if (!this.isHandshake) {
            this.messagesBeforeHandshake.add(message);
            return;
        }
        this.channel.writeAndFlush(message).addListener(result::handle);
    }

    @FunctionalInterface
    public interface Result {
        void handle(Future<?> future) throws InterruptedException;
    }
}
