# Scientific Data Provenance Policy

Each bundled material/environment record must contain:
- source_id and canonical citation
- source URL/DOI
- access/version date
- original units and normalized SI units
- nominal value plus range/uncertainty where available
- temperature/pressure/moisture/process conditions
- whether value is measured, calculated, fitted or inferred
- license/redistribution status
- transformation/calibration notes
- game approximation tier

## Licensing warning
Do not bulk-copy NIST Standard Reference Data merely because it is publicly viewable. NIST states SRD may be copyrighted/licensed. Prefer non-SRD public NIST works where possible, store derived/calibrated game parameters when legally appropriate, and obtain/verify permission for bundled SRD datasets.

Materials Project is an open resource and provides APIs, but bulk collection/dissemination has usage/attribution rules. Treat API data as provenance-bearing input, not as a database to mirror blindly.

Open-source code licenses also matter:
- Apache/MIT components are easiest to integrate.
- GPL projects such as KeeperRL/Veloren/OpenTTD are excellent architecture references, but copying/linking code can impose distribution obligations. Keep benchmark notes separate from implementation unless license compatibility is explicitly approved.
