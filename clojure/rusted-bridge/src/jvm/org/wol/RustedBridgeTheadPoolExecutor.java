class RustedBridgeThreadPoolExecutor extends OrderedMemoryAwareThreadPoolExecutor {

     @Override
     protected ConcurrentMap<Object, Executor> newChildExecutorMap() {
         // The default implementation returns a special ConcurrentMap that
         // uses identity comparison only (see IdentityHashMap).
         // Because SocketAddress does not work with identity comparison,
         // we need to employ more generic implementation.
         return new ConcurrentHashMap<Object, Executor>
     }

     protected Object getChildExecutorKey(ChannelEvent e) {
         // Use the IP of the remote peer as a key.
         return ((InetSocketAddress) e.getChannel().getRemoteAddress()).getAddress();
     }

     // Make public so that you can call from anywhere.
     public boolean removeChildExecutor(Object key) {
         super.removeChildExecutor(key);
     }
}
