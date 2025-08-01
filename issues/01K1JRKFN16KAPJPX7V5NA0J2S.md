The place where you error log.

Detected ABORT ERROR in prompt output, triggering immediate shutdown

REALLY -- I keep saying this -- hard exit with a non zero exit code. There is no reason to continue afterward.

REALLY immediate shutdown, DO NOT KEEP GOING when running.

I know this makes the tests tricky, so have conditional behavior when running in test.