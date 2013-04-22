(ns org.wol.clj-grep.core
  (:require [org.wol.rusted-bridge.server :as server]
            [org.wol.rusted-bridge.commands :as commands])
  (:use [clojure.tools.cli :only [cli]])
  (:gen-class))

(defn -main [& args]
  (let [[matched-args [pattern & garbage] help-doc]
        (cli args ["-s" "--[no-]server" "start bridge server" :default false])]
    (if (:server matched-args)      
      (server/start-bridge :dispatch-fn -main :port 9001)
      (let [pattern (re-pattern pattern)]
        (loop []
          (when-let [next-line (read-line)]
            (if (re-find pattern next-line)
              (println next-line))            
            (recur)))))))



(comment

  (def server (server/start-bridge :dispatch-fn -main :port 9001))
  (server/stop-server server)


  (def burger ( atom []))
  (doseq [a @burger]
    (if (re-find (re-pattern "screen") a)
      (println a)))
  
  )