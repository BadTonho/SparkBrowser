import test from "node:test";
import assert from "node:assert/strict";
import { addHibernationDomain, hibernationDomainMatches, normalizeHibernationDomain } from "../src/lib/services/hibernation-domains.js";

test("normalizes hostname case, trailing dots, and IDNs", () => {
  assert.equal(normalizeHibernationDomain(" Example.COM. "), "example.com");
  assert.equal(normalizeHibernationDomain("münich.example"), "xn--mnich-kva.example");
});

test("matches apex and subdomains at label boundaries", () => {
  assert.equal(hibernationDomainMatches("shop.example.com", ["example.com"]), true);
  assert.equal(hibernationDomainMatches("example.com", ["example.com"]), true);
  assert.equal(hibernationDomainMatches("notexample.com", ["example.com"]), false);
  assert.equal(hibernationDomainMatches("example.com.attacker.test", ["example.com"]), false);
});

test("rejects invalid entries and deduplicates additions", () => {
  assert.equal(normalizeHibernationDomain("https://example.com/path"), null);
  assert.deepEqual(addHibernationDomain("EXAMPLE.com.", ["example.com"]), null);
  assert.deepEqual(addHibernationDomain("shop.example.com", ["example.com"]), ["example.com", "shop.example.com"]);
});
