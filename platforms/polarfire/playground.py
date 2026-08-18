# LED / UART hooks for the session agent.
# Writes one JSON object per line to HWSTATE_PATH (default /tmp/hartlab-hw.jsonl).
# Renode IronPython 2: no f-strings, no pathlib.
#
# Phase 0: fail loudly. A silent bind leaves the board dark and the agent
# looking healthy.

import json
import os

_hw_path = os.environ.get("HWSTATE_PATH", "/tmp/hartlab-hw.jsonl")
_leds = [False, False, False, False]
_running = True

LED_PATHS = (
    ("sysbus.gpio2.led1", 0),
    ("sysbus.gpio2.led2", 1),
    ("sysbus.gpio2.led3", 2),
    ("sysbus.gpio2.led4", 3),
)


def _emit(extra=None):
    payload = {
        "type": "hw",
        "running": _running,
        "leds": list(_leds),
    }
    if extra:
        payload.update(extra)
    handle = open(_hw_path, "a")
    try:
        handle.write(json.dumps(payload) + "\n")
    finally:
        handle.close()


def _on_led(index, state):
    _leds[index] = bool(state)
    _emit()


def mc_hw_set_running(flag):
    global _running
    _running = str(flag).lower() not in ("0", "false", "False")
    _emit()


def _bind_led(path, index):
    try:
        led = monitor.Machine[path]
    except Exception, ex:
        raise Exception("HartLab: cannot resolve LED %s (%s)" % (path, ex))
    if not hasattr(led, "StateChanged"):
        raise Exception(
            "HartLab: %s has no StateChanged — GPIO2 LED overlay did not attach" % path
        )
    def _handler(sender, state, idx=index):
        _on_led(idx, state)
    led.StateChanged += _handler
    print "HartLab: bound %s" % path


for path, index in LED_PATHS:
    _bind_led(path, index)
_emit()
print "HartLab: LED hooks armed, writing %s" % _hw_path
