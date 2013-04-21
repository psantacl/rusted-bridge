(ns org.wol.clj-ls.core
  (:require [org.wol.rusted-bridge.server :as server]
            [org.wol.rusted-bridge.commands :as commands])
  (:use [clojure.tools.cli :only [cli]])
  (:gen-class))

(defn -main [& args]
  (let [[matched-args [target-file & garbage] help-doc]
        (cli args ["-s" "--[no-]server" "start bridge server" :default false])]
     (if (:server matched-args)      
      (server/start-bridge :dispatch-fn -main)
      (doseq [next-file (-> (java.io.File.  target-file)
                            (.list))]
        (println next-file)))))


(comment

  (cli ["/tmp/"] ["-s" "--[no-]server" "start bridge server" :default false])
  
  (def server (server/start-bridge :dispatch-fn -main))
  (server/stop-server server)

  (println "what is your name?")           
  (let [name (read-line)]
    (println (format "hi there %s" name)))
  (.getName (first (file-seq (java.io.File. "/tmp/"))))
)












