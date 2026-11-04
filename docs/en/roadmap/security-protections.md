# Planned protection against suspicious sites and downloads

Status: **planned, not implemented**. This is the second option discussed during the internal-security work. The first delivery reinforces application boundaries and WebView2 update awareness.

A future delivery may add phishing and malicious-site reputation checks, warnings for suspicious downloads and explicit user decisions before continuing. Before implementing it, define:

- Which reputation provider or local lists to use, their license, coverage, availability and cost.
- What URLs, hashes or other information would leave the device, retention policies and user consent.
- How to handle unavailable services, false positives, appeals and temporary exceptions.
- Which detections produce warnings, which block actions, and how decisions are explained.
- How cached results and lists are refreshed, authenticated and tested.

No reputation service is contacted by the current security-hardening delivery. Keep this proposal separate from the existing privacy shield and file-checksum verification.
