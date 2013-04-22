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
          (do
            (println next-line)
            (recur (inc i)))
          (println i))))))



(comment
  (def burger (atom []))
  (def server (server/start-bridge :dispatch-fn -main :port 9002))
  (server/stop-server server)

  )