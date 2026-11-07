import assert from 'node:assert/strict';
export async function checkFeeds({ invoke, fixture, restart }) {
  assert.deepEqual(await invoke('get_feeds'), []);
  await invoke('save_feed', { id: null, title: 'Fixture', url: `${fixture.origin}/fixture-feed.xml`, intervalMinutes: 60 });
  const [feed] = await invoke('get_feeds');
  await invoke('refresh_feed', { id: feed.id });
  const [entry] = await invoke('get_feed_entries', { id: feed.id });
  assert.equal(entry.title, 'Fixture article');
  assert.equal(entry.url, `${fixture.origin}/page/feed-article`);
  assert.equal(entry.text, '<script>not executed</script>');
  fixture.feed.invalid = true;
  await assert.rejects(invoke('refresh_feed', { id: feed.id }));
  assert.equal((await invoke('get_feed_entries', { id: feed.id })).length, 1);
  assert.ok((await invoke('get_feeds'))[0].last_error);
  assert.deepEqual(await invoke('get_experimental_flags'), { compact_feed_cards: false });
  await invoke('set_experimental_flag', { name: 'compact_feed_cards', enabled: true });
  await restart();
  assert.equal((await invoke('get_feeds')).length, 1);
  assert.equal((await invoke('get_feed_entries', { id: feed.id })).length, 1);
  assert.equal((await invoke('get_experimental_flags')).compact_feed_cards, true);
  await invoke('reset_experimental_flags');
  assert.equal((await invoke('get_experimental_flags')).compact_feed_cards, false);
  await invoke('delete_feed', { id: feed.id });
  assert.deepEqual(await invoke('get_feeds'), []);
}
