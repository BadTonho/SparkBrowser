import { runScenario } from '../../tests/native/harness.mjs';
import { checkYoutubeAutoSkip } from '../../tests/native/auto-skip-youtube.mjs';
await runScenario('auto-skip-youtube', checkYoutubeAutoSkip);
