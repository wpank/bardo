// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="introduction/what-is-bardo.html">Introduction</a></span></li><li class="chapter-item expanded "><li class="part-title">Technical Documentation</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="architecture/workspace.html"><strong aria-hidden="true">1.</strong> Workspace</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-core.html"><strong aria-hidden="true">2.</strong> Crates</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-runtime.html"><strong aria-hidden="true">2.1.</strong> golem-runtime</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-heartbeat.html"><strong aria-hidden="true">2.2.</strong> golem-heartbeat</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-grimoire.html"><strong aria-hidden="true">2.3.</strong> golem-grimoire</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-daimon.html"><strong aria-hidden="true">2.4.</strong> golem-daimon</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-mortality.html"><strong aria-hidden="true">2.5.</strong> golem-mortality</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-dreams.html"><strong aria-hidden="true">2.6.</strong> golem-dreams</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-context.html"><strong aria-hidden="true">2.7.</strong> golem-context</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-safety.html"><strong aria-hidden="true">2.8.</strong> golem-safety</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-inference.html"><strong aria-hidden="true">2.9.</strong> golem-inference</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-chain.html"><strong aria-hidden="true">2.10.</strong> golem-chain</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-chain-intelligence.html"><strong aria-hidden="true">2.11.</strong> golem-chain-intelligence</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-triage.html"><strong aria-hidden="true">2.12.</strong> golem-triage</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-ta.html"><strong aria-hidden="true">2.13.</strong> golem-ta</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-oneirography.html"><strong aria-hidden="true">2.14.</strong> golem-oneirography</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-tools.html"><strong aria-hidden="true">2.15.</strong> golem-tools</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-coordination.html"><strong aria-hidden="true">2.16.</strong> golem-coordination</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-surfaces.html"><strong aria-hidden="true">2.17.</strong> golem-surfaces</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-creature.html"><strong aria-hidden="true">2.18.</strong> golem-creature</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-engagement.html"><strong aria-hidden="true">2.19.</strong> golem-engagement</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="crates/golem-binary.html"><strong aria-hidden="true">2.20.</strong> golem-binary</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="apps/bardo-gateway.html"><strong aria-hidden="true">3.</strong> Apps</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="apps/bardo-terminal.html"><strong aria-hidden="true">3.1.</strong> bardo-terminal</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="apps/bardo-styx.html"><strong aria-hidden="true">3.2.</strong> bardo-styx</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="apps/bardo-compute.html"><strong aria-hidden="true">3.3.</strong> bardo-compute</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="apps/mirage-rs.html"><strong aria-hidden="true">3.4.</strong> mirage-rs</a></span></li></ol><li class="chapter-item expanded "><li class="part-title">Specification</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/SUMMARY.html"><strong aria-hidden="true">4.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/00-narrative-strategy.html"><strong aria-hidden="true">5.</strong> Narrative Strategy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/00-vision/00-bardo.html"><strong aria-hidden="true">6.</strong> Vision</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/00-vision/01-thesis.html"><strong aria-hidden="true">6.1.</strong> Thesis</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/00-vision/02-architecture.html"><strong aria-hidden="true">6.2.</strong> Architecture</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/00-vision/03-philosophy.html"><strong aria-hidden="true">6.3.</strong> Philosophy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/00-vision/04-trust.html"><strong aria-hidden="true">6.4.</strong> Trust</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/00-vision/05-manifesto.html"><strong aria-hidden="true">6.5.</strong> Manifesto</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/00-overview.html"><strong aria-hidden="true">7.</strong> Golem</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/01-cognition.html"><strong aria-hidden="true">7.1.</strong> Cognition</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/02-heartbeat.html"><strong aria-hidden="true">7.2.</strong> Heartbeat</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/03-mind.html"><strong aria-hidden="true">7.3.</strong> Mind</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/03b-cognitive-mechanisms.html"><strong aria-hidden="true">7.4.</strong> Cognitive Mechanisms</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/03c-state-management.html"><strong aria-hidden="true">7.5.</strong> State Management</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/04-mortality.html"><strong aria-hidden="true">7.6.</strong> Mortality</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/05-death.html"><strong aria-hidden="true">7.7.</strong> Death</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/06-creation.html"><strong aria-hidden="true">7.8.</strong> Creation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/07-provisioning.html"><strong aria-hidden="true">7.9.</strong> Provisioning</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/08-funding.html"><strong aria-hidden="true">7.10.</strong> Funding</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/09-inheritance.html"><strong aria-hidden="true">7.11.</strong> Inheritance</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/10-replication.html"><strong aria-hidden="true">7.12.</strong> Replication</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/11-lifecycle.html"><strong aria-hidden="true">7.13.</strong> Lifecycle</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/12-teardown.html"><strong aria-hidden="true">7.14.</strong> Teardown</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/13-runtime-extensions.html"><strong aria-hidden="true">7.15.</strong> Runtime Extensions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/13a-runtime-extensions.html"><strong aria-hidden="true">7.16.</strong> Runtime Extensions A</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/13b-runtime-extensions.html"><strong aria-hidden="true">7.17.</strong> Runtime Extensions B</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/14-context-governor.html"><strong aria-hidden="true">7.18.</strong> Context Governor</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/14b-attention-auction.html"><strong aria-hidden="true">7.19.</strong> Attention Auction</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/15-sleepwalker.html"><strong aria-hidden="true">7.20.</strong> Sleepwalker</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/16-risk-engine.html"><strong aria-hidden="true">7.21.</strong> Risk Engine</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/17-prediction-engine.html"><strong aria-hidden="true">7.22.</strong> Prediction Engine</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/17b-ta-prediction-domains.html"><strong aria-hidden="true">7.23.</strong> TA Prediction Domains</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/18-cortical-state.html"><strong aria-hidden="true">7.24.</strong> Cortical State</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/01-golem/19-config-and-operator-model.html"><strong aria-hidden="true">7.25.</strong> Config &amp; Operator Model</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/00-thesis.html"><strong aria-hidden="true">8.</strong> Mortality</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/01-architecture.html"><strong aria-hidden="true">8.1.</strong> Architecture</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/02-epistemic-decay.html"><strong aria-hidden="true">8.2.</strong> Epistemic Decay</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/03-stochastic-mortality.html"><strong aria-hidden="true">8.3.</strong> Stochastic Mortality</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/04-economic-mortality.html"><strong aria-hidden="true">8.4.</strong> Economic Mortality</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/05-knowledge-demurrage.html"><strong aria-hidden="true">8.5.</strong> Knowledge Demurrage</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/06-thanatopsis.html"><strong aria-hidden="true">8.6.</strong> Thanatopsis</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/07-succession.html"><strong aria-hidden="true">8.7.</strong> Succession</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/08-mortality-affect.html"><strong aria-hidden="true">8.8.</strong> Mortality Affect</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/09-fractal-mortality.html"><strong aria-hidden="true">8.9.</strong> Fractal Mortality</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/10-clade-ecology.html"><strong aria-hidden="true">8.10.</strong> Clade Ecology</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/10b-morphogenetic-specialization.html"><strong aria-hidden="true">8.11.</strong> Morphogenetic Specialization</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/11-immortal-control.html"><strong aria-hidden="true">8.12.</strong> Immortal Control</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/12-integration.html"><strong aria-hidden="true">8.13.</strong> Integration</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/13-configuration.html"><strong aria-hidden="true">8.14.</strong> Configuration</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/14-research-foundations.html"><strong aria-hidden="true">8.15.</strong> Research Foundations</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/15-references.html"><strong aria-hidden="true">8.16.</strong> References</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/16-necrocracy.html"><strong aria-hidden="true">8.17.</strong> Necrocracy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/17-information-theoretic-diagnostics.html"><strong aria-hidden="true">8.18.</strong> Information-Theoretic Diagnostics</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/02-mortality/18-antifragile-mortality.html"><strong aria-hidden="true">8.19.</strong> Antifragile Mortality</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/00-overview.html"><strong aria-hidden="true">9.</strong> Daimon</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/01-appraisal.html"><strong aria-hidden="true">9.1.</strong> Appraisal</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/02-emotion-memory.html"><strong aria-hidden="true">9.2.</strong> Emotion Memory</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/03-behavior.html"><strong aria-hidden="true">9.3.</strong> Behavior</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/04-mortality-daimon.html"><strong aria-hidden="true">9.4.</strong> Mortality Daimon</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/05-death-daimon.html"><strong aria-hidden="true">9.5.</strong> Death Daimon</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/06-dream-daimon.html"><strong aria-hidden="true">9.6.</strong> Dream Daimon</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/07-runtime-daimon.html"><strong aria-hidden="true">9.7.</strong> Runtime Daimon</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/08-infrastructure.html"><strong aria-hidden="true">9.8.</strong> Infrastructure</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/03-daimon/09-evaluation.html"><strong aria-hidden="true">9.9.</strong> Evaluation</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/00-overview.html"><strong aria-hidden="true">10.</strong> Memory</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/01-grimoire.html"><strong aria-hidden="true">10.1.</strong> Grimoire</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/01b-grimoire-memetic.html"><strong aria-hidden="true">10.2.</strong> Grimoire Memetic</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/01c-grimoire-hdc.html"><strong aria-hidden="true">10.3.</strong> Grimoire HDC</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/02-emotional-memory.html"><strong aria-hidden="true">10.4.</strong> Emotional Memory</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/03-mortal-memory.html"><strong aria-hidden="true">10.5.</strong> Mortal Memory</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/06-economy.html"><strong aria-hidden="true">10.6.</strong> Economy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/09-safety.html"><strong aria-hidden="true">10.7.</strong> Safety</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/10-research.html"><strong aria-hidden="true">10.8.</strong> Research</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/11-roadmap.html"><strong aria-hidden="true">10.9.</strong> Roadmap</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/12-katabasis.html"><strong aria-hidden="true">10.10.</strong> Katabasis</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/04-memory/13-library-of-babel.html"><strong aria-hidden="true">10.11.</strong> Library of Babel</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/05-dreams/00-overview.html"><strong aria-hidden="true">11.</strong> Dreams</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/05-dreams/01-architecture.html"><strong aria-hidden="true">11.1.</strong> Architecture</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/05-dreams/01b-dream-evolution.html"><strong aria-hidden="true">11.2.</strong> Dream Evolution</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/05-dreams/02-replay.html"><strong aria-hidden="true">11.3.</strong> Replay</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/05-dreams/03-imagination.html"><strong aria-hidden="true">11.4.</strong> Imagination</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/05-dreams/04-consolidation.html"><strong aria-hidden="true">11.5.</strong> Consolidation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/05-dreams/05-threats.html"><strong aria-hidden="true">11.6.</strong> Threats</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/05-dreams/06-integration.html"><strong aria-hidden="true">11.7.</strong> Integration</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/05-dreams/07-venice-dreaming.html"><strong aria-hidden="true">11.8.</strong> Venice Dreaming</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/06-hypnagogia/00-overview.html"><strong aria-hidden="true">12.</strong> Hypnagogia</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/06-hypnagogia/01-neuroscience.html"><strong aria-hidden="true">12.1.</strong> Neuroscience</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/06-hypnagogia/02-architecture.html"><strong aria-hidden="true">12.2.</strong> Architecture</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/06-hypnagogia/03-divergence-alpha.html"><strong aria-hidden="true">12.3.</strong> Divergence Alpha</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/06-hypnagogia/04-homunculus.html"><strong aria-hidden="true">12.4.</strong> Homunculus</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/06-hypnagogia/05-hauntology.html"><strong aria-hidden="true">12.5.</strong> Hauntology</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/06-hypnagogia/06-xenocognition.html"><strong aria-hidden="true">12.6.</strong> Xenocognition</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/06-hypnagogia/07-inner-worlds.html"><strong aria-hidden="true">12.7.</strong> Inner Worlds</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/00-overview.html"><strong aria-hidden="true">13.</strong> Tools</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/01-architecture.html"><strong aria-hidden="true">13.1.</strong> Architecture</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/02-tools-data.html"><strong aria-hidden="true">13.2.</strong> Data</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/03-tools-trading.html"><strong aria-hidden="true">13.3.</strong> Trading</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/04-tools-lp.html"><strong aria-hidden="true">13.4.</strong> Liquidity</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/05-tools-bridge-aggregator.html"><strong aria-hidden="true">13.5.</strong> Bridge &amp; Aggregator</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/06-tools-vault.html"><strong aria-hidden="true">13.6.</strong> Vault</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/07-tools-lending.html"><strong aria-hidden="true">13.7.</strong> Lending</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/08-tools-staking.html"><strong aria-hidden="true">13.8.</strong> Staking</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/09-tools-restaking.html"><strong aria-hidden="true">13.9.</strong> Restaking</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/10-tools-derivatives.html"><strong aria-hidden="true">13.10.</strong> Derivatives</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/11-tools-yield.html"><strong aria-hidden="true">13.11.</strong> Yield</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/12-tools-safety.html"><strong aria-hidden="true">13.12.</strong> Safety</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/13-tools-intelligence.html"><strong aria-hidden="true">13.13.</strong> Intelligence</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/14-tools-identity.html"><strong aria-hidden="true">13.14.</strong> Identity</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/15-tools-memory.html"><strong aria-hidden="true">13.15.</strong> Memory</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/16-tools-testnet.html"><strong aria-hidden="true">13.16.</strong> Testnet</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/17-tools-uniswap-api.html"><strong aria-hidden="true">13.17.</strong> Uniswap API</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/18-tools-metamask.html"><strong aria-hidden="true">13.18.</strong> MetaMask</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/19-tools-streaming.html"><strong aria-hidden="true">13.19.</strong> Streaming</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/20-config.html"><strong aria-hidden="true">13.20.</strong> Config</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/21-profiles.html"><strong aria-hidden="true">13.21.</strong> Profiles</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/22-wallets.html"><strong aria-hidden="true">13.22.</strong> Wallets</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/23-distribution.html"><strong aria-hidden="true">13.23.</strong> Distribution</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/07-tools/24-testing.html"><strong aria-hidden="true">13.24.</strong> Testing</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/09-economy/00-identity.html"><strong aria-hidden="true">14.</strong> Economy</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/09-economy/01-reputation.html"><strong aria-hidden="true">14.1.</strong> Reputation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/09-economy/02-clade.html"><strong aria-hidden="true">14.2.</strong> Clade</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/09-economy/03-marketplace.html"><strong aria-hidden="true">14.3.</strong> Marketplace</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/09-economy/04-coordination.html"><strong aria-hidden="true">14.4.</strong> Coordination</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/09-economy/05-agent-economy.html"><strong aria-hidden="true">14.5.</strong> Agent Economy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/09-economy/06-commerce-bazaar.html"><strong aria-hidden="true">14.6.</strong> Commerce Bazaar</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/00-defense.html"><strong aria-hidden="true">15.</strong> Safety</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/01-custody.html"><strong aria-hidden="true">15.1.</strong> Custody</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/02-policy.html"><strong aria-hidden="true">15.2.</strong> Policy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/03-ingestion.html"><strong aria-hidden="true">15.3.</strong> Ingestion</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/04-prompt-security.html"><strong aria-hidden="true">15.4.</strong> Prompt Security</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/05-threat-model.html"><strong aria-hidden="true">15.5.</strong> Threat Model</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/06-adaptive-risk.html"><strong aria-hidden="true">15.6.</strong> Adaptive Risk</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/07-temporal-logic-verification.html"><strong aria-hidden="true">15.7.</strong> Temporal Logic Verification</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/08-witness-dag.html"><strong aria-hidden="true">15.8.</strong> Witness DAG</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/09-formal-verification-pipeline.html"><strong aria-hidden="true">15.9.</strong> Formal Verification Pipeline</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/10-safety/10-mev-protection.html"><strong aria-hidden="true">15.10.</strong> MEV Protection</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/11-compute/00-overview.html"><strong aria-hidden="true">16.</strong> Compute</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/11-compute/01-architecture.html"><strong aria-hidden="true">16.1.</strong> Architecture</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/11-compute/02-provisioning.html"><strong aria-hidden="true">16.2.</strong> Provisioning</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/11-compute/03-billing.html"><strong aria-hidden="true">16.3.</strong> Billing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/11-compute/04-security.html"><strong aria-hidden="true">16.4.</strong> Security</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/11-compute/05-operations.html"><strong aria-hidden="true">16.5.</strong> Operations</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/11-compute/06-api.html"><strong aria-hidden="true">16.6.</strong> API</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/11-compute/07-frontend.html"><strong aria-hidden="true">16.7.</strong> Frontend</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/11-compute/08-fleet-and-cli.html"><strong aria-hidden="true">16.8.</strong> Fleet &amp; CLI</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/00-overview.html"><strong aria-hidden="true">17.</strong> Inference</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/01-deployment-modes.html"><strong aria-hidden="true">17.1.</strong> Deployment Modes</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/01a-routing.html"><strong aria-hidden="true">17.2.</strong> Routing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/02-caching.html"><strong aria-hidden="true">17.3.</strong> Caching</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/03-economics.html"><strong aria-hidden="true">17.4.</strong> Economics</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/04-context-engineering.html"><strong aria-hidden="true">17.5.</strong> Context Engineering</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/05-sessions.html"><strong aria-hidden="true">17.6.</strong> Sessions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/06-memory.html"><strong aria-hidden="true">17.7.</strong> Memory</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/07-safety.html"><strong aria-hidden="true">17.8.</strong> Safety</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/08-observability.html"><strong aria-hidden="true">17.9.</strong> Observability</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/09-api.html"><strong aria-hidden="true">17.10.</strong> API</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/10-roadmap.html"><strong aria-hidden="true">17.11.</strong> Roadmap</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/11-privacy-trust.html"><strong aria-hidden="true">17.12.</strong> Privacy &amp; Trust</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/12-providers.html"><strong aria-hidden="true">17.13.</strong> Providers</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/13-reasoning.html"><strong aria-hidden="true">17.14.</strong> Reasoning</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/14-rust-implementation.html"><strong aria-hidden="true">17.15.</strong> Rust Implementation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/15-inference-profiles.html"><strong aria-hidden="true">17.16.</strong> Inference Profiles</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/16-structured-outputs.html"><strong aria-hidden="true">17.17.</strong> Structured Outputs</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/17-streaming.html"><strong aria-hidden="true">17.18.</strong> Streaming</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/18-golem-config.html"><strong aria-hidden="true">17.19.</strong> Golem Config</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/19-multi-model-orchestration.html"><strong aria-hidden="true">17.20.</strong> Multi-Model Orchestration</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/20-inference-parameters.html"><strong aria-hidden="true">17.21.</strong> Inference Parameters</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/21-inference-performance.html"><strong aria-hidden="true">17.22.</strong> Inference Performance</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/12-inference/sheaf-observation.html"><strong aria-hidden="true">17.23.</strong> Sheaf Observation</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/00-interaction-model.html"><strong aria-hidden="true">18.</strong> Runtime</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/01-defi-activities.html"><strong aria-hidden="true">18.1.</strong> DeFi Activities</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/02-communication-channels.html"><strong aria-hidden="true">18.2.</strong> Communication Channels</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/03-auth-access-control.html"><strong aria-hidden="true">18.3.</strong> Auth &amp; Access Control</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/04-data-visibility.html"><strong aria-hidden="true">18.4.</strong> Data Visibility</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/05-knowledge-browser.html"><strong aria-hidden="true">18.5.</strong> Knowledge Browser</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/06-collective-intelligence.html"><strong aria-hidden="true">18.6.</strong> Collective Intelligence</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/07-onboarding.html"><strong aria-hidden="true">18.7.</strong> Onboarding</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/08-public-data-gateway.html"><strong aria-hidden="true">18.8.</strong> Public Data Gateway</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/09-observability.html"><strong aria-hidden="true">18.9.</strong> Observability</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/10-packaging-deployment.html"><strong aria-hidden="true">18.10.</strong> Packaging &amp; Deployment</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/11-state-model.html"><strong aria-hidden="true">18.11.</strong> State Model</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/12-realtime-subscriptions.html"><strong aria-hidden="true">18.12.</strong> Realtime Subscriptions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/13-engagement-loops.html"><strong aria-hidden="true">18.13.</strong> Engagement Loops</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/14-creature-system.html"><strong aria-hidden="true">18.14.</strong> Creature System</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/15-progression-meta.html"><strong aria-hidden="true">18.15.</strong> Progression Meta</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/16-social-competitive.html"><strong aria-hidden="true">18.16.</strong> Social &amp; Competitive</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/17-platform-ux.html"><strong aria-hidden="true">18.17.</strong> Platform UX</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/18-retention-virality.html"><strong aria-hidden="true">18.18.</strong> Retention &amp; Virality</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/19-cinematic-system.html"><strong aria-hidden="true">18.19.</strong> Cinematic System</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/20-solaris.html"><strong aria-hidden="true">18.20.</strong> Solaris</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/21-cybernetic-loops.html"><strong aria-hidden="true">18.21.</strong> Cybernetic Loops</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/13-runtime/22-first-fifteen-minutes.html"><strong aria-hidden="true">18.22.</strong> First Fifteen Minutes</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/00-architecture.html"><strong aria-hidden="true">19.</strong> Chain</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/01-witness.html"><strong aria-hidden="true">19.1.</strong> Witness</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/02-triage.html"><strong aria-hidden="true">19.2.</strong> Triage</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/03-protocol-state.html"><strong aria-hidden="true">19.3.</strong> Protocol State</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/04-chain-scope.html"><strong aria-hidden="true">19.4.</strong> Chain Scope</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/05-heartbeat-integration.html"><strong aria-hidden="true">19.5.</strong> Heartbeat Integration</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/06-events-signals.html"><strong aria-hidden="true">19.6.</strong> Events &amp; Signals</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/07-generative-views.html"><strong aria-hidden="true">19.7.</strong> Generative Views</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/08-stream-api.html"><strong aria-hidden="true">19.8.</strong> Stream API</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/14-chain/09-anomaly-detection.html"><strong aria-hidden="true">19.9.</strong> Anomaly Detection</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/00-overview.html"><strong aria-hidden="true">20.</strong> Development</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/01-mirage-rs.html"><strong aria-hidden="true">20.1.</strong> Mirage RS</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/01b-mirage-rpc.html"><strong aria-hidden="true">20.2.</strong> Mirage RPC</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/01c-mirage-scenarios.html"><strong aria-hidden="true">20.3.</strong> Mirage Scenarios</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/01d-mirage-integration.html"><strong aria-hidden="true">20.4.</strong> Mirage Integration</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/01e-mirage-tx-compatibility.html"><strong aria-hidden="true">20.5.</strong> Mirage TX Compatibility</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/02-deployment.html"><strong aria-hidden="true">20.6.</strong> Deployment</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/03-debug-ui.html"><strong aria-hidden="true">20.7.</strong> Debug UI</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/04-scenarios.html"><strong aria-hidden="true">20.8.</strong> Scenarios</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/05-tooling.html"><strong aria-hidden="true">20.9.</strong> Tooling</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/15-dev/06-indexer.html"><strong aria-hidden="true">20.10.</strong> Indexer</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/00-thesis-validation.html"><strong aria-hidden="true">21.</strong> Testing</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/01-gauntlet.html"><strong aria-hidden="true">21.1.</strong> Gauntlet</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/02-knowledge-quality.html"><strong aria-hidden="true">21.2.</strong> Knowledge Quality</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/03-mechanism-testing.html"><strong aria-hidden="true">21.3.</strong> Mechanism Testing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/04-mirage.html"><strong aria-hidden="true">21.4.</strong> Mirage</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/05-evaluation-lifecycle.html"><strong aria-hidden="true">21.5.</strong> Evaluation Lifecycle</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/06-revision-guide.html"><strong aria-hidden="true">21.6.</strong> Revision Guide</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/07-fast-feedback-loops.html"><strong aria-hidden="true">21.7.</strong> Fast Feedback Loops</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/08-slow-feedback-loops.html"><strong aria-hidden="true">21.8.</strong> Slow Feedback Loops</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/09-evaluation-map.html"><strong aria-hidden="true">21.9.</strong> Evaluation Map</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/10-retrospective-evaluation.html"><strong aria-hidden="true">21.10.</strong> Retrospective Evaluation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/11-mirage-v2-testing.html"><strong aria-hidden="true">21.11.</strong> Mirage V2 Testing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/12-simulation-validation.html"><strong aria-hidden="true">21.12.</strong> Simulation Validation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/13-triage-evaluation.html"><strong aria-hidden="true">21.13.</strong> Triage Evaluation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/14-chain-scope-testing.html"><strong aria-hidden="true">21.14.</strong> Chain Scope Testing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/16-testing/15-tui-testing.html"><strong aria-hidden="true">21.15.</strong> TUI Testing</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/17-monorepo/00-packages.html"><strong aria-hidden="true">22.</strong> Monorepo</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/17-monorepo/01-rust-workspace.html"><strong aria-hidden="true">22.1.</strong> Rust Workspace</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/17-monorepo/02-build.html"><strong aria-hidden="true">22.2.</strong> Build</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/17-monorepo/03-conventions.html"><strong aria-hidden="true">22.3.</strong> Conventions</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/00-portal.html"><strong aria-hidden="true">23.</strong> Interfaces</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/01-cli.html"><strong aria-hidden="true">23.1.</strong> CLI</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/02-ui-system.html"><strong aria-hidden="true">23.2.</strong> UI System</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/03-tui.html"><strong aria-hidden="true">23.3.</strong> TUI</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/19-spatial-grammar.html"><strong aria-hidden="true">23.4.</strong> Spatial Grammar</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/26-bardo-terminal-foundation.html"><strong aria-hidden="true">23.5.</strong> Terminal Foundation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/28-creature-system.html"><strong aria-hidden="true">23.6.</strong> Creature System</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/perspective/00-nooscopy.html"><strong aria-hidden="true">23.7.</strong> Perspective</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/perspective/01-golem-perspective.html"><strong aria-hidden="true">23.7.1.</strong> Golem Perspective</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/perspective/02-portals.html"><strong aria-hidden="true">23.7.2.</strong> Portals</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/perspective/03-embodied-consciousness.html"><strong aria-hidden="true">23.7.3.</strong> Embodied Consciousness</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/perspective/04-inner-worlds.html"><strong aria-hidden="true">23.7.4.</strong> Inner Worlds</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/perspective/05-stasis-dissolution.html"><strong aria-hidden="true">23.7.5.</strong> Stasis Dissolution</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/perspective/06-hauntology.html"><strong aria-hidden="true">23.7.6.</strong> Hauntology</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/protocol/00-sanctum-protocol-layer.html"><strong aria-hidden="true">23.8.</strong> Protocol</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/protocol/01-protocol-view-catalog.html"><strong aria-hidden="true">23.8.1.</strong> Protocol View Catalog</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/protocol/02-generative-views.html"><strong aria-hidden="true">23.8.2.</strong> Generative Views</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/rendering/00-design-system.html"><strong aria-hidden="true">23.9.</strong> Rendering</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/rendering/01-demoscene.html"><strong aria-hidden="true">23.9.1.</strong> Demoscene</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/rendering/02-visualization-primitives.html"><strong aria-hidden="true">23.9.2.</strong> Visualization Primitives</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/rendering/03-transitions.html"><strong aria-hidden="true">23.9.3.</strong> Transitions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/rendering/04-nerv-aesthetic.html"><strong aria-hidden="true">23.9.4.</strong> NERV Aesthetic</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/screens/00-screen-catalog.html"><strong aria-hidden="true">23.10.</strong> Screens</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/screens/01-screen-specs.html"><strong aria-hidden="true">23.10.1.</strong> Screen Specs</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/screens/02-widget-catalog.html"><strong aria-hidden="true">23.10.2.</strong> Widget Catalog</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/screens/03-interaction-hierarchy.html"><strong aria-hidden="true">23.10.3.</strong> Interaction Hierarchy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/screens/04-oracle-surfaces.html"><strong aria-hidden="true">23.10.4.</strong> Oracle Surfaces</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/18-interfaces/screens/05-math-metaphor.html"><strong aria-hidden="true">23.10.5.</strong> Math Metaphor</a></span></li></ol></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/00-agents-overview.html"><strong aria-hidden="true">24.</strong> Agents &amp; Skills</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/01-agent-categories.html"><strong aria-hidden="true">24.1.</strong> Agent Categories</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/02-agent-definitions.html"><strong aria-hidden="true">24.2.</strong> Agent Definitions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/03-delegation.html"><strong aria-hidden="true">24.3.</strong> Delegation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/04-skills-overview.html"><strong aria-hidden="true">24.4.</strong> Skills Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/05-skill-categories.html"><strong aria-hidden="true">24.5.</strong> Skill Categories</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/06-skill-definitions.html"><strong aria-hidden="true">24.6.</strong> Skill Definitions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/08-mcp-integration.html"><strong aria-hidden="true">24.7.</strong> MCP Integration</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/09-golem-agents.html"><strong aria-hidden="true">24.8.</strong> Golem Agents</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/10-vault-agents.html"><strong aria-hidden="true">24.9.</strong> Vault Agents</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/11-composition.html"><strong aria-hidden="true">24.10.</strong> Composition</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/12-observer-agents.html"><strong aria-hidden="true">24.11.</strong> Observer Agents</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/19-agents-skills/13-hermes-hierarchy.html"><strong aria-hidden="true">24.12.</strong> Hermes Hierarchy</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/20-styx/00-architecture.html"><strong aria-hidden="true">25.</strong> Styx</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/20-styx/01-api.html"><strong aria-hidden="true">25.1.</strong> API</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/20-styx/02-infrastructure.html"><strong aria-hidden="true">25.2.</strong> Infrastructure</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/20-styx/03-clade-sync.html"><strong aria-hidden="true">25.3.</strong> Clade Sync</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/20-styx/04-marketplace.html"><strong aria-hidden="true">25.4.</strong> Marketplace</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/20-styx/05-tui-experience.html"><strong aria-hidden="true">25.5.</strong> TUI Experience</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/20-styx/06-deployment.html"><strong aria-hidden="true">25.6.</strong> Deployment</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/21-integrations/00-overview.html"><strong aria-hidden="true">26.</strong> Integrations</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/21-integrations/01-metamask.html"><strong aria-hidden="true">26.1.</strong> MetaMask</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/21-integrations/02-venice.html"><strong aria-hidden="true">26.2.</strong> Venice</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/21-integrations/03-bankr.html"><strong aria-hidden="true">26.3.</strong> Bankr</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/21-integrations/04-agentcash.html"><strong aria-hidden="true">26.4.</strong> AgentCash</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/21-integrations/05-uniswap.html"><strong aria-hidden="true">26.5.</strong> Uniswap</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/22-oneirography/00-overview.html"><strong aria-hidden="true">27.</strong> Oneirography</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/22-oneirography/01-dream-journals.html"><strong aria-hidden="true">27.1.</strong> Dream Journals</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/22-oneirography/02-death-masks.html"><strong aria-hidden="true">27.2.</strong> Death Masks</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/22-oneirography/03-self-appraisal.html"><strong aria-hidden="true">27.3.</strong> Self-Appraisal</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/22-oneirography/04-auctions.html"><strong aria-hidden="true">27.4.</strong> Auctions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/22-oneirography/05-extended-forms.html"><strong aria-hidden="true">27.5.</strong> Extended Forms</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/22-oneirography/06-contracts.html"><strong aria-hidden="true">27.6.</strong> Contracts</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/22-oneirography/07-gallery-tui.html"><strong aria-hidden="true">27.7.</strong> Gallery TUI</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/00-witness-as-technical-analyst.html"><strong aria-hidden="true">28.</strong> Technical Analysis</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/01-hyperdimensional-technical-analysis.html"><strong aria-hidden="true">28.1.</strong> Hyperdimensional TA</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/02-spectral-liquidity-manifolds.html"><strong aria-hidden="true">28.2.</strong> Spectral Liquidity Manifolds</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/03-adaptive-signal-metabolism.html"><strong aria-hidden="true">28.3.</strong> Adaptive Signal Metabolism</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/04-causal-microstructure-discovery.html"><strong aria-hidden="true">28.4.</strong> Causal Microstructure Discovery</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/05-predictive-geometry.html"><strong aria-hidden="true">28.5.</strong> Predictive Geometry</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/06-resonant-pattern-ecosystem.html"><strong aria-hidden="true">28.6.</strong> Resonant Pattern Ecosystem</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/07-defi-native-technical-analysis.html"><strong aria-hidden="true">28.7.</strong> DeFi-Native TA</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/08-adversarial-signal-robustness.html"><strong aria-hidden="true">28.8.</strong> Adversarial Signal Robustness</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/09-somatic-technical-analysis.html"><strong aria-hidden="true">28.9.</strong> Somatic TA</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/23-ta/10-emergent-multiscale-intelligence.html"><strong aria-hidden="true">28.10.</strong> Emergent Multiscale Intelligence</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/24-sonification/00-overview.html"><strong aria-hidden="true">29.</strong> Sonification</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/24-sonification/01-module-system.html"><strong aria-hidden="true">29.1.</strong> Module System</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/24-sonification/02-cortical-mapping.html"><strong aria-hidden="true">29.2.</strong> Cortical Mapping</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/24-sonification/03-terminal-rack.html"><strong aria-hidden="true">29.3.</strong> Terminal Rack</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/24-sonification/04-nft-state.html"><strong aria-hidden="true">29.4.</strong> NFT State</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/24-sonification/05-musical-language.html"><strong aria-hidden="true">29.5.</strong> Musical Language</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/24-sonification/06-preset-catalog.html"><strong aria-hidden="true">29.6.</strong> Preset Catalog</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/25-mori/mori-overview.html"><strong aria-hidden="true">30.</strong> Mori</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/25-mori/mori-agent-architecture.html"><strong aria-hidden="true">30.1.</strong> Agent Architecture</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/25-mori/mori-context-engineering.html"><strong aria-hidden="true">30.2.</strong> Context Engineering</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/25-mori/mori-cost-efficiency.html"><strong aria-hidden="true">30.3.</strong> Cost Efficiency</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/25-mori/mori-parallel-execution.html"><strong aria-hidden="true">30.4.</strong> Parallel Execution</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/25-mori/mori-quality-gates.html"><strong aria-hidden="true">30.5.</strong> Quality Gates</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/25-mori/mori-resilience.html"><strong aria-hidden="true">30.6.</strong> Resilience</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/25-mori/mori-unified-dag.html"><strong aria-hidden="true">30.7.</strong> Unified DAG</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/appendices/a-life-in-numbers.html"><strong aria-hidden="true">31.</strong> Appendices</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/appendices/competitive-analysis.html"><strong aria-hidden="true">31.1.</strong> Competitive Analysis</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/appendices/dying-machine.html"><strong aria-hidden="true">31.2.</strong> The Dying Machine</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/appendices/implementation-state.html"><strong aria-hidden="true">31.3.</strong> Implementation State</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/appendices/market-context.html"><strong aria-hidden="true">31.4.</strong> Market Context</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/appendices/performance-targets.html"><strong aria-hidden="true">31.5.</strong> Performance Targets</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/glossary.html"><strong aria-hidden="true">32.</strong> Shared Reference</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/branding.html"><strong aria-hidden="true">32.1.</strong> Branding</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/chains.html"><strong aria-hidden="true">32.2.</strong> Chains</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/citations.html"><strong aria-hidden="true">32.3.</strong> Citations</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/config-reference.html"><strong aria-hidden="true">32.4.</strong> Config Reference</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/data-privacy.html"><strong aria-hidden="true">32.5.</strong> Data Privacy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/dependencies.html"><strong aria-hidden="true">32.6.</strong> Dependencies</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/doc-standards.html"><strong aria-hidden="true">32.7.</strong> Doc Standards</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/eip-analysis.html"><strong aria-hidden="true">32.8.</strong> EIP Analysis</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/emergent-capabilities.html"><strong aria-hidden="true">32.9.</strong> Emergent Capabilities</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/evaluation.html"><strong aria-hidden="true">32.10.</strong> Evaluation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/event-catalog.html"><strong aria-hidden="true">32.11.</strong> Event Catalog</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/hdc-applications.html"><strong aria-hidden="true">32.12.</strong> HDC Applications</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/hdc-fingerprints.html"><strong aria-hidden="true">32.13.</strong> HDC Fingerprints</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/hdc-vsa.html"><strong aria-hidden="true">32.14.</strong> HDC VSA</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/integrated-information.html"><strong aria-hidden="true">32.15.</strong> Integrated Information</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/port-allocation.html"><strong aria-hidden="true">32.16.</strong> Port Allocation</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/research.html"><strong aria-hidden="true">32.17.</strong> Research</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/timeline.html"><strong aria-hidden="true">32.18.</strong> Timeline</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="prd/shared/x402-protocol.html"><strong aria-hidden="true">32.19.</strong> x402 Protocol</a></span></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split('#')[0].split('?')[0];
        if (current_page.endsWith('/')) {
            current_page += 'index.html';
        }
        const links = Array.prototype.slice.call(this.querySelectorAll('a'));
        const l = links.length;
        for (let i = 0; i < l; ++i) {
            const link = links[i];
            const href = link.getAttribute('href');
            if (href && !href.startsWith('#') && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The 'index' page is supposed to alias the first chapter in the book.
            if (link.href === current_page
                || i === 0
                && path_to_root === ''
                && current_page.endsWith('/index.html')) {
                link.classList.add('active');
                let parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', e => {
            if (e.target.tagName === 'A') {
                const clientRect = e.target.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                sessionStorage.setItem('sidebar-scroll-offset', clientRect.top - sidebarRect.top);
            }
        }, { passive: true });
        const sidebarScrollOffset = sessionStorage.getItem('sidebar-scroll-offset');
        sessionStorage.removeItem('sidebar-scroll-offset');
        if (sidebarScrollOffset !== null) {
            // preserve sidebar scroll position when navigating via links within sidebar
            const activeSection = this.querySelector('.active');
            if (activeSection) {
                const clientRect = activeSection.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                const currentOffset = clientRect.top - sidebarRect.top;
                this.scrollTop += currentOffset - parseFloat(sidebarScrollOffset);
            }
        } else {
            // scroll sidebar to current active section when navigating via
            // 'next/previous chapter' buttons
            const activeSection = document.querySelector('#mdbook-sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        const sidebarAnchorToggles = document.querySelectorAll('.chapter-fold-toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(el => {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define('mdbook-sidebar-scrollbox', MDBookSidebarScrollbox);


// ---------------------------------------------------------------------------
// Support for dynamically adding headers to the sidebar.

(function() {
    // This is used to detect which direction the page has scrolled since the
    // last scroll event.
    let lastKnownScrollPosition = 0;
    // This is the threshold in px from the top of the screen where it will
    // consider a header the "current" header when scrolling down.
    const defaultDownThreshold = 150;
    // Same as defaultDownThreshold, except when scrolling up.
    const defaultUpThreshold = 300;
    // The threshold is a virtual horizontal line on the screen where it
    // considers the "current" header to be above the line. The threshold is
    // modified dynamically to handle headers that are near the bottom of the
    // screen, and to slightly offset the behavior when scrolling up vs down.
    let threshold = defaultDownThreshold;
    // This is used to disable updates while scrolling. This is needed when
    // clicking the header in the sidebar, which triggers a scroll event. It
    // is somewhat finicky to detect when the scroll has finished, so this
    // uses a relatively dumb system of disabling scroll updates for a short
    // time after the click.
    let disableScroll = false;
    // Array of header elements on the page.
    let headers;
    // Array of li elements that are initially collapsed headers in the sidebar.
    // I'm not sure why eslint seems to have a false positive here.
    // eslint-disable-next-line prefer-const
    let headerToggles = [];
    // This is a debugging tool for the threshold which you can enable in the console.
    let thresholdDebug = false;

    // Updates the threshold based on the scroll position.
    function updateThreshold() {
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const windowHeight = window.innerHeight;
        const documentHeight = document.documentElement.scrollHeight;

        // The number of pixels below the viewport, at most documentHeight.
        // This is used to push the threshold down to the bottom of the page
        // as the user scrolls towards the bottom.
        const pixelsBelow = Math.max(0, documentHeight - (scrollTop + windowHeight));
        // The number of pixels above the viewport, at least defaultDownThreshold.
        // Similar to pixelsBelow, this is used to push the threshold back towards
        // the top when reaching the top of the page.
        const pixelsAbove = Math.max(0, defaultDownThreshold - scrollTop);
        // How much the threshold should be offset once it gets close to the
        // bottom of the page.
        const bottomAdd = Math.max(0, windowHeight - pixelsBelow - defaultDownThreshold);
        let adjustedBottomAdd = bottomAdd;

        // Adjusts bottomAdd for a small document. The calculation above
        // assumes the document is at least twice the windowheight in size. If
        // it is less than that, then bottomAdd needs to be shrunk
        // proportional to the difference in size.
        if (documentHeight < windowHeight * 2) {
            const maxPixelsBelow = documentHeight - windowHeight;
            const t = 1 - pixelsBelow / Math.max(1, maxPixelsBelow);
            const clamp = Math.max(0, Math.min(1, t));
            adjustedBottomAdd *= clamp;
        }

        let scrollingDown = true;
        if (scrollTop < lastKnownScrollPosition) {
            scrollingDown = false;
        }

        if (scrollingDown) {
            // When scrolling down, move the threshold up towards the default
            // downwards threshold position. If near the bottom of the page,
            // adjustedBottomAdd will offset the threshold towards the bottom
            // of the page.
            const amountScrolledDown = scrollTop - lastKnownScrollPosition;
            const adjustedDefault = defaultDownThreshold + adjustedBottomAdd;
            threshold = Math.max(adjustedDefault, threshold - amountScrolledDown);
        } else {
            // When scrolling up, move the threshold down towards the default
            // upwards threshold position. If near the bottom of the page,
            // quickly transition the threshold back up where it normally
            // belongs.
            const amountScrolledUp = lastKnownScrollPosition - scrollTop;
            const adjustedDefault = defaultUpThreshold - pixelsAbove
                + Math.max(0, adjustedBottomAdd - defaultDownThreshold);
            threshold = Math.min(adjustedDefault, threshold + amountScrolledUp);
        }

        if (documentHeight <= windowHeight) {
            threshold = 0;
        }

        if (thresholdDebug) {
            const id = 'mdbook-threshold-debug-data';
            let data = document.getElementById(id);
            if (data === null) {
                data = document.createElement('div');
                data.id = id;
                data.style.cssText = `
                    position: fixed;
                    top: 50px;
                    right: 10px;
                    background-color: 0xeeeeee;
                    z-index: 9999;
                    pointer-events: none;
                `;
                document.body.appendChild(data);
            }
            data.innerHTML = `
                <table>
                  <tr><td>documentHeight</td><td>${documentHeight.toFixed(1)}</td></tr>
                  <tr><td>windowHeight</td><td>${windowHeight.toFixed(1)}</td></tr>
                  <tr><td>scrollTop</td><td>${scrollTop.toFixed(1)}</td></tr>
                  <tr><td>pixelsAbove</td><td>${pixelsAbove.toFixed(1)}</td></tr>
                  <tr><td>pixelsBelow</td><td>${pixelsBelow.toFixed(1)}</td></tr>
                  <tr><td>bottomAdd</td><td>${bottomAdd.toFixed(1)}</td></tr>
                  <tr><td>adjustedBottomAdd</td><td>${adjustedBottomAdd.toFixed(1)}</td></tr>
                  <tr><td>scrollingDown</td><td>${scrollingDown}</td></tr>
                  <tr><td>threshold</td><td>${threshold.toFixed(1)}</td></tr>
                </table>
            `;
            drawDebugLine();
        }

        lastKnownScrollPosition = scrollTop;
    }

    function drawDebugLine() {
        if (!document.body) {
            return;
        }
        const id = 'mdbook-threshold-debug-line';
        const existingLine = document.getElementById(id);
        if (existingLine) {
            existingLine.remove();
        }
        const line = document.createElement('div');
        line.id = id;
        line.style.cssText = `
            position: fixed;
            top: ${threshold}px;
            left: 0;
            width: 100vw;
            height: 2px;
            background-color: red;
            z-index: 9999;
            pointer-events: none;
        `;
        document.body.appendChild(line);
    }

    function mdbookEnableThresholdDebug() {
        thresholdDebug = true;
        updateThreshold();
        drawDebugLine();
    }

    window.mdbookEnableThresholdDebug = mdbookEnableThresholdDebug;

    // Updates which headers in the sidebar should be expanded. If the current
    // header is inside a collapsed group, then it, and all its parents should
    // be expanded.
    function updateHeaderExpanded(currentA) {
        // Add expanded to all header-item li ancestors.
        let current = currentA.parentElement;
        while (current) {
            if (current.tagName === 'LI' && current.classList.contains('header-item')) {
                current.classList.add('expanded');
            }
            current = current.parentElement;
        }
    }

    // Updates which header is marked as the "current" header in the sidebar.
    // This is done with a virtual Y threshold, where headers at or below
    // that line will be considered the current one.
    function updateCurrentHeader() {
        if (!headers || !headers.length) {
            return;
        }

        // Reset the classes, which will be rebuilt below.
        const els = document.getElementsByClassName('current-header');
        for (const el of els) {
            el.classList.remove('current-header');
        }
        for (const toggle of headerToggles) {
            toggle.classList.remove('expanded');
        }

        // Find the last header that is above the threshold.
        let lastHeader = null;
        for (const header of headers) {
            const rect = header.getBoundingClientRect();
            if (rect.top <= threshold) {
                lastHeader = header;
            } else {
                break;
            }
        }
        if (lastHeader === null) {
            lastHeader = headers[0];
            const rect = lastHeader.getBoundingClientRect();
            const windowHeight = window.innerHeight;
            if (rect.top >= windowHeight) {
                return;
            }
        }

        // Get the anchor in the summary.
        const href = '#' + lastHeader.id;
        const a = [...document.querySelectorAll('.header-in-summary')]
            .find(element => element.getAttribute('href') === href);
        if (!a) {
            return;
        }

        a.classList.add('current-header');

        updateHeaderExpanded(a);
    }

    // Updates which header is "current" based on the threshold line.
    function reloadCurrentHeader() {
        if (disableScroll) {
            return;
        }
        updateThreshold();
        updateCurrentHeader();
    }


    // When clicking on a header in the sidebar, this adjusts the threshold so
    // that it is located next to the header. This is so that header becomes
    // "current".
    function headerThresholdClick(event) {
        // See disableScroll description why this is done.
        disableScroll = true;
        setTimeout(() => {
            disableScroll = false;
        }, 100);
        // requestAnimationFrame is used to delay the update of the "current"
        // header until after the scroll is done, and the header is in the new
        // position.
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                // Closest is needed because if it has child elements like <code>.
                const a = event.target.closest('a');
                const href = a.getAttribute('href');
                const targetId = href.substring(1);
                const targetElement = document.getElementById(targetId);
                if (targetElement) {
                    threshold = targetElement.getBoundingClientRect().bottom;
                    updateCurrentHeader();
                }
            });
        });
    }

    // Takes the nodes from the given head and copies them over to the
    // destination, along with some filtering.
    function filterHeader(source, dest) {
        const clone = source.cloneNode(true);
        clone.querySelectorAll('mark').forEach(mark => {
            mark.replaceWith(...mark.childNodes);
        });
        dest.append(...clone.childNodes);
    }

    // Scans page for headers and adds them to the sidebar.
    document.addEventListener('DOMContentLoaded', function() {
        const activeSection = document.querySelector('#mdbook-sidebar .active');
        if (activeSection === null) {
            return;
        }

        const main = document.getElementsByTagName('main')[0];
        headers = Array.from(main.querySelectorAll('h2, h3, h4, h5, h6'))
            .filter(h => h.id !== '' && h.children.length && h.children[0].tagName === 'A');

        if (headers.length === 0) {
            return;
        }

        // Build a tree of headers in the sidebar.

        const stack = [];

        const firstLevel = parseInt(headers[0].tagName.charAt(1));
        for (let i = 1; i < firstLevel; i++) {
            const ol = document.createElement('ol');
            ol.classList.add('section');
            if (stack.length > 0) {
                stack[stack.length - 1].ol.appendChild(ol);
            }
            stack.push({level: i + 1, ol: ol});
        }

        // The level where it will start folding deeply nested headers.
        const foldLevel = 3;

        for (let i = 0; i < headers.length; i++) {
            const header = headers[i];
            const level = parseInt(header.tagName.charAt(1));

            const currentLevel = stack[stack.length - 1].level;
            if (level > currentLevel) {
                // Begin nesting to this level.
                for (let nextLevel = currentLevel + 1; nextLevel <= level; nextLevel++) {
                    const ol = document.createElement('ol');
                    ol.classList.add('section');
                    const last = stack[stack.length - 1];
                    const lastChild = last.ol.lastChild;
                    // Handle the case where jumping more than one nesting
                    // level, which doesn't have a list item to place this new
                    // list inside of.
                    if (lastChild) {
                        lastChild.appendChild(ol);
                    } else {
                        last.ol.appendChild(ol);
                    }
                    stack.push({level: nextLevel, ol: ol});
                }
            } else if (level < currentLevel) {
                while (stack.length > 1 && stack[stack.length - 1].level > level) {
                    stack.pop();
                }
            }

            const li = document.createElement('li');
            li.classList.add('header-item');
            li.classList.add('expanded');
            if (level < foldLevel) {
                li.classList.add('expanded');
            }
            const span = document.createElement('span');
            span.classList.add('chapter-link-wrapper');
            const a = document.createElement('a');
            span.appendChild(a);
            a.href = '#' + header.id;
            a.classList.add('header-in-summary');
            filterHeader(header.children[0], a);
            a.addEventListener('click', headerThresholdClick);
            const nextHeader = headers[i + 1];
            if (nextHeader !== undefined) {
                const nextLevel = parseInt(nextHeader.tagName.charAt(1));
                if (nextLevel > level && level >= foldLevel) {
                    const toggle = document.createElement('a');
                    toggle.classList.add('chapter-fold-toggle');
                    toggle.classList.add('header-toggle');
                    toggle.addEventListener('click', () => {
                        li.classList.toggle('expanded');
                    });
                    const toggleDiv = document.createElement('div');
                    toggleDiv.textContent = '❱';
                    toggle.appendChild(toggleDiv);
                    span.appendChild(toggle);
                    headerToggles.push(li);
                }
            }
            li.appendChild(span);

            const currentParent = stack[stack.length - 1];
            currentParent.ol.appendChild(li);
        }

        const onThisPage = document.createElement('div');
        onThisPage.classList.add('on-this-page');
        onThisPage.append(stack[0].ol);
        const activeItemSpan = activeSection.parentElement;
        activeItemSpan.after(onThisPage);
    });

    document.addEventListener('DOMContentLoaded', reloadCurrentHeader);
    document.addEventListener('scroll', reloadCurrentHeader, { passive: true });
})();

