# Error Handling Flowcharts (ASCII)

## Runtime Missing
```txt
Error -> runtime missing
  -> UI shows download action
    -> user accepts
      -> runtime.ensureService
        -> success -> retry start
        -> failure -> show error + diagnostics
```

## Port Conflict
```txt
Error -> port conflict
  -> UI offers auto-assign
    -> apply new port -> retry start
  -> manual override -> save -> retry start
```

## Config Invalid
```txt
Error -> config invalid
  -> show file path + open config
  -> offer reset to defaults
  -> validate -> retry start
```

## Update Signature Invalid
```txt
Error -> signature invalid
  -> block update
  -> offer retry + contact support
```
