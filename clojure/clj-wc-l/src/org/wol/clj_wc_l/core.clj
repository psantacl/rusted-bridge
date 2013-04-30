(ns org.wol.clj-wc-l.core
  (:require [org.wol.rusted-bridge.server :as server]
            [org.wol.rusted-bridge.commands :as commands])
  (:use [clojure.tools.cli :only [cli]])
  (:gen-class))

(defn -main [& args]
  (let [[matched-args [pattern & garbage] help-doc]
        (cli args ["-s" "--[no-]server" "start bridge server" :default false])]
    (if (:server matched-args)      
      (server/start-bridge :dispatch-fn -main :port 9002)
      (loop [i 0]
        (if-let [next-line (read-line)]
          (recur (inc i))
          (println i))))))

(comment

  (def out-pipe       (java.io.PipedOutputStream. ))
  (def in-pipe        (java.io.PipedInputStream. out-pipe))
  (def std-in-thread  (java.util.concurrent.Executors/newSingleThreadExecutor))
  
  (defn make-callalble [out-pipe in-payload-data]
    (reify  java.util.concurrent.Callable
      (call [self]
        (let [nascent-data (.getBytes in-payload-data "UTF-8")]
          (.write out-pipe nascent-data 0 (count nascent-data))))))

  (def *shit*  (java.io.BufferedReader. (java.io.InputStreamReader. in-pipe "UTF-8")))
  (binding [*in* *shit*]
    (loop []
      (println "reading...")
      (if-let [next-line  (read-line)]
        (do
          (println (format "next_line: %s" next-line))
          (recur))
        (println "all done"))))

  (.close in-pipe)
  (.close out-pipe) ;;YES!
  (.close *shit*)
  (.submit std-in-thread (make-callalble out-pipe "chicken!\n"))  


  (def server (server/start-bridge :dispatch-fn -main :port 9002))
  (server/stop-server server)

  )