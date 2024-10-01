get_tests: tests tests/nestest.nes tests/nestest.log

tests:
	mkdir tests

tests/nestest.nes:
	curl https://www.qmtpro.com/~nes/misc/nestest.nes > tests/nestest.nes

tests/nestest.log: tests/ref_nestest.log
	cat tests/ref_nestest.log | sed -E "s/^(....).{44}A:(..) X:(..) Y:(..) P:(..) SP:(..) PPU:\s*([0-9]+),\s*([0-9]+) CYC:([^\r]*)\r?$$/\L\1 \2 \3 \4 \5 \6 \9 \7 \8/g" > tests/nestest.log

tests/ref_nestest.log:
	curl https://www.qmtpro.com/~nes/misc/nestest.log > tests/ref_nestest.log

.PHONY: get_tests
