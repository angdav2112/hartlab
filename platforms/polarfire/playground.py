# LED / UART hooks for the session agent.
# Writes one JSON object per line to HWSTATE_PATH (default /tmp/hartlab-hw.jsonl).
# Renode IronPython 2: no f-strings, no pathlib.

import json
import os

_hw_path = os.environ.get("HWSTATE_PATH", "/tmp/hartlab-hw.jsonl")
_leds = [False, False, False, False]
_running = True


def _emit(extra=None):
    payload = {
        "type": "hw",
        "running": _running,
        "leds": list(_leds),
    }
    if extra:
        payload.update(extra)
    try:
        handle = open(_hw_path, "a")
        handle.write(json.dumps(payload) + "\n")
        handle.close()
    except Exception:
        pass


def _on_led(index, state):
    _leds[index] = bool(state)
    _emit()


def mc_hw_set_running(flag):
    global _running
    _running = str(flag).lower() not in ("0", "false", "False")
    _emit()


def _bind_led(path, index):
    # IronPython: resolve the LED and subscribe if StateChanged exists.
    try:
        led = monitor.Machine[path]
    except Exception:
        return
    try:
        def _handler(sender, state, idx=index):
            _on_led(idx, state)

        led.StateChanged += _handler
    except Exception:
        pass


_bind_led("sysbus.gpio2.led1", 0)
_bind_led("sysbus.gpio2.led2", 1)
_bind_led("sysbus.gpio2.led3", 2)
_bind_led("sysbus.gpio2.led4", 3)
_emit()
