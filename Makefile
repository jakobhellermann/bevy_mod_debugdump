all:
	cd debugdumpgen && make
	cd web && npm run build

.PHONY: docs
docs:
	cd web && npm run build

clean:
	cd debugdumpgen && make clean
	cd web && rm -rf .parcel-cache
	rm -rf docs
