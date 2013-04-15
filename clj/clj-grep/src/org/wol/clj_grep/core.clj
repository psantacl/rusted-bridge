(ns org.wol.clj-grep.core
  (:require [rusted-bridge.server :as server]
            [rusted-bridge.commands :as commands])
  (:use [clojure.tools.cli :only [cli]])
  (:gen-class))

(defn -main [& args]
  (let [[matched-args [pattern & garbage] help-doc]
        (cli args ["-s" "--[no-]server" "start bridge server" :default false])]
    (if (:server matched-args)      
      (server/start-bridge :dispatch-fn -main)
      (let [pattern (re-pattern pattern)]
        (loop []
          (when-let [next-line (read-line)]
            (if (re-find pattern next-line)
              (println next-line))
            (recur )))))))

(comment

  (def server (server/start-bridge :dispatch-fn -main))
  (server/stop-server server)

  )