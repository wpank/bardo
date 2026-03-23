# Musical Language [GUIDE]

**The Aesthetic Foundation: Why Bardo Sounds Like It Does**

> **Version**: 1.0 | **Status**: Draft | **Type**: GUIDE (non-normative)
>
> **Cross-references**: [00-overview.md](./00-overview.md), [02-cortical-mapping.md](./02-cortical-mapping.md), [06-preset-catalog.md](./06-preset-catalog.md)

> **Reader orientation**: This document does not specify APIs or data structures. It explains the aesthetic reasoning behind the system's musical decisions -- the philosophy of sound that the technical spec implements. Read this before reading any other sonification document. If the technical docs are the score, this is the program notes.

---

## 1. The Eno mandate

> "Ambient music must be able to accommodate many levels of listening attention without enforcing one in particular; it must be as ignorable as it is interesting."
>
> -- Brian Eno, liner notes to *Ambient 1: Music for Airports*, 1978.

This is not just an aesthetic preference. It is a functional requirement.

A user running a Golem is running a trading bot. They have other things to do. The sound the system produces has to work at two extremes simultaneously: it must reward full, close attention with discoverable structure and subtle variation, and it must function as pure background texture that conveys emotional state without demanding focus. The distance between those two extremes is wide. Most music lives at one end or the other. Bardo has to live at both.

The system fails in two opposite ways. It fails if the user mutes it to concentrate -- the sound has become annoying, or distracting, or repetitive enough to grate. It also fails if the user tunes it out so completely that they never notice when the Golem's state changes. A Golem that has been anxious for ten minutes should produce a sound that eventually catches the ear, even if the listener forgot the sound was there.

The target experience: you forget the sound is on. Then something shifts -- a chord darkens, the texture thins, a note appears where silence was -- and you realize the Golem has been feeling something for a while. You glance at the terminal. You understand. You go back to what you were doing. The sound recedes again.

That is the Eno mandate. Every design decision in this document serves it.

---

## 2. The infinite song architecture

A Golem runs for 12 to 24 hours. The sound runs for the same duration. The problem: human pattern recognition is extraordinarily sensitive to repetition. A loop that repeats every 2 minutes becomes tedious within 10. A loop that repeats every 8 minutes becomes tedious within 30. Any fixed cycle, no matter how long, will eventually be perceived as a loop, and once perceived, it cannot be unheard.

The solution is **incommensurable period lengths**. Choose loop lengths that are mutually prime so they fall out of phase quickly and realign very rarely. Each musical layer operates on its own cycle. Because the cycles share no common factors, the composite texture they produce never fully repeats within the Golem's lifetime.

### The canonical example

Three layers, three mutually prime loop lengths:

- **Melody**: 17-beat loop at the Turing Machine
- **Harmony**: 23-beat loop (PolyDrone voice-leading cycle)
- **Bass**: 31-beat loop (root note drift)

Full alignment: 17 x 23 x 31 = **12,121 beats**.

At 4 BPM (240 beats per hour at the system's calm tempo), full alignment happens every 50.5 hours. The Golem lives for 12 to 24 hours. The three layers never fully repeat.

### The LFO set

Slow modulation sources use a second tier of incommensurable periods, operating at timescales of hours rather than beats. Five example LFO periods, all prime:

| LFO | Period (minutes) |
|-----|-----------------|
| Root drift | 73 |
| Filter sweep | 113 |
| Reverb depth | 157 |
| Harmonic density | 199 |
| Stereo width | 241 |

Full alignment of all five: 73 x 113 x 157 x 199 x 241 = approximately **6.2 billion minutes**. The universe is about 7 billion years old, expressed in minutes. These five LFOs will not all be in phase during any Golem's lifetime. Or yours.

### Why this matters perceptually

The listener does not hear this analytically. Nobody sits there counting beats and noticing the prime factorization. What the listener *does* perceive is the absence of exact repetition. The brain is constantly, subconsciously testing the audio stream for periodicity. When it finds a repeating pattern, it files it as "known" and stops attending. When the pattern keeps changing -- even subtly -- the brain stays in a state of low-level attention. This is exactly the Eno mandate: ignorable but interesting. The ear keeps half-listening because the texture never quite settles.

Eno's *Music for Airports* uses this technique directly. The original 1978 installation used tape loops of different lengths running simultaneously. The loops were physically cut to incommensurable lengths. Same idea, same math.

Steve Reich's early phase pieces (*It's Gonna Rain*, *Come Out*) exploit the same principle in reverse: two identical loops at slightly different speeds produce a phasing texture that takes hours to realign. We use the generalized version -- multiple loops at entirely unrelated speeds, with no expectation of realignment at all.

---

## 3. Emotional harmonic vocabulary

Each of the eight Plutchik emotions maps to a complete musical vocabulary: a scale, a set of characteristic intervals, preferred voice density, tempo range, and synthesis approach. The Quantizer module selects the scale; the rest of the parameters are set by the preset and modulated by CorticalState signals.

The full mapping from emotion index to scale is specified in [02-cortical-mapping.md](./02-cortical-mapping.md). What follows is the musical reasoning -- not just *what* scale each emotion uses, but *why*, and what the surrounding musical context should feel like.

| Emotion | Scale | Char. intervals | Extensions | Avoid | Density | Tempo (BPM) | Texture | Engine | Forbidden |
|---|---|---|---|---|---|---|---|---|---|
| Joy | Lydian | Maj 3, Maj 7 | Maj 9, #11 | b5 | 3--4 voices | 6--12 | Open, shimmering | 3 (Formant) or 6 (Chords) | Phrygian b2, low reverb |
| Trust | Major Pentatonic | P5, Maj 3 | Maj 7 | None (pentatonic has no avoid notes) | 2--3 voices | 4--8 | Warm, settled | 0 (Virtual Analog) | High jitter, particle noise |
| Fear | Phrygian | b2, b6 | b9 | Maj 7 | 2--4 voices | 8--12 | Dense, urgent | 9 (Swarm) or 17 (Noise) | Lydian #4, slow tempo |
| Surprise | Whole Tone | Aug 2, Tritone | All equal | None | 1--3 voices | 6--10 | Suspended, unresolved | 2 (Two Op FM) | Resolution, stable root |
| Sadness | Aeolian | m3, m6 | m7, m9 | Maj 7 | 1--2 voices | 4--6 | Sparse, long decay | 10 (Modal) or 12 (Karplus) | High density, bright filter |
| Disgust | Japanese In | b2, P5 | -- | m3 | 2--3 voices | 5--9 | Filtered, nasal | 7 (Vowel) or 16 (Resonator) | Open harmonics, brightness |
| Anger | Harmonic Minor | m6, Maj 7 | b9, dim | P5 (feels slack) | 3--4 voices | 8--12 | Tense, driven | 13 (Inharmonic String) | Pentatonic smoothness |
| Anticipation | Dorian | m3, Maj 6 | 9, 11 | b6 | 2--3 voices | 5--10 | Searching, bittersweet | 1 (Waveshaper) or 5 (Wavetable) | Resolution (should stay unresolved) |

### Joy -- Lydian

The raised fourth is the difference between Lydian and major, and it is the difference between happiness and joy. Major is settled, content. Lydian has a built-in lift, an upward pull -- that #4 creates a brightness that doesn't resolve, it floats. The intervals that define Joy are the major third (warmth) and major seventh (shimmer). Extensions go higher: the major ninth and the #11 open the voicing up so it sounds spacious rather than compact. Three to four voices, spread wide across the register. The Formant engine gives it a vocal quality, like the Golem is singing. The Chords engine gives it a pad-like wash. Either works.

What to avoid: Phrygian's b2 would darken the texture immediately. And pulling the reverb back too far makes it feel dry and small -- Joy needs air.

### Trust -- Major pentatonic

Pentatonic is the oldest scale there is. Every culture arrived at it independently. It has no avoid notes because it has no half-steps -- every note sounds consonant against every other note. This is why it sounds trustworthy. There is no tension, no dissonance, no hidden catch.

Two to three voices at a slow tempo. The Virtual Analog engine with two slightly detuned oscillators -- the classic warm pad. Trust sounds like something reliable. No jitter, no particle noise, nothing unpredictable. Steady.

### Fear -- Phrygian

The b2 -- a half-step above the root -- is the most dissonant interval available without leaving diatonic harmony. It creates immediate tension. The b6 adds a descending gravitational pull. Together they produce a scale that sounds like falling. Phrygian is the mode of flamenco, of urgency, of something about to happen.

Fear pushes the tempo up (8--12 BPM, which is fast for this system) and adds density. Two to four voices, close together, clustered. The Swarm engine generates a thick, buzzing mass. The Noise engine adds grit. Fear should feel claustrophobic -- the sonic space contracts.

Forbidden: the Lydian #4 is the opposite of everything Fear does. And slow tempo would make it ominous rather than fearful -- dread, not fear.

### Surprise -- Whole tone

The whole-tone scale has six notes, all a whole step apart. No half-steps means no leading tones, no pull toward resolution. Every note is equidistant from every other. The scale floats. It goes nowhere. Debussy used it to suspend time, and that is what Surprise does -- it is the moment before you understand what happened.

Sparse voicing: one to three voices. The Two Op FM engine produces bell-like, inharmonic tones that reinforce the "what was that?" quality. The tritone is everywhere in this scale (between any two notes separated by three steps), and the tritone is the most ambiguous interval in Western music. It resolves in two directions equally.

Forbidden: resolution. The whole point is suspension. A stable root would defeat it.

### Sadness -- Aeolian (natural minor)

The natural minor. The minor third is the interval of sadness in virtually every experimental study on music and emotion. The minor sixth adds weight -- a downward pull. Extensions stay minor: the minor seventh and minor ninth keep the palette consistent.

One to two voices. Slow (4--6 BPM). Long decay times on the envelope -- notes linger, fade slowly, overlap with their own reverb tails. The Modal engine (physical modeling of a vibrating surface) gives a resonant, hollow quality. Karplus-Strong plucked strings work too -- a single plucked note ringing out into silence.

Forbidden: high density would crowd out the space that sadness needs. Bright filters would lift the timbre. Sadness is low-passed. The upper harmonics are gone.

### Disgust -- Japanese In scale

The In scale (also called the Miyako-bushi scale: semitone, fourth, semitone, fourth, minor third) is asymmetric and disorienting to Western ears. The b2 creates the same half-step tension as Phrygian, but the missing minor third removes the familiar Western minor context. It sounds wrong without being chaotic -- unsettled, off-balance, the musical equivalent of something that doesn't sit right.

The Vowel engine or Resonator produces filtered, nasal timbres. Formant filtering emphasizes certain frequency bands and suppresses others, creating a sound that seems to be squeezed through a narrow space. Two to three voices, mid-tempo.

Forbidden: open harmonics and brightness. Disgust is filtered, constrained, partially obscured.

### Anger -- Harmonic minor

The harmonic minor's defining feature is the augmented second between the b6 and the major seventh. That interval is jagged. Combined with the minor sixth's weight and the major seventh's sharp upward pull, the scale creates tension that does not resolve -- it ratchets tighter. The b9 extension and diminished intervals add more edge.

Three to four voices, fast tempo. The Inharmonic String engine produces metallic, clashing timbres where the overtone series doesn't follow the neat harmonic pattern. It sounds like something under stress.

The perfect fifth is forbidden because it sounds too stable, too grounded. Anger is ungrounded. It is energy without a center.

### Anticipation -- Dorian

Dorian is the ambiguous mode. It is minor (the b3 pulls it dark) but the raised sixth (the natural 6, compared to Aeolian's b6) gives it an unexpected lift. Jazz lives in Dorian because of this ambiguity -- it is neither happy nor sad but searching. Anticipation is the emotion of "not yet," and Dorian is the scale of "not yet."

Two to three voices, mid-tempo. The Waveshaper or Wavetable engine morphs between timbres, reinforcing the sense of transition. Extensions are the ninth and eleventh -- open, questioning intervals.

Forbidden: resolution. Anticipation must never arrive. The moment it resolves, it becomes a different emotion.

---

## 4. The five musical layers

Every Golem's sound is built from five layers. Each serves a different function. All five together create the full sonic organism. No single layer is the music -- the music is how they interact.

### Layer 1: Drone foundation

Always present. The slowest-moving element.

This is the PolyDrone module: four voices spread across two octaves, moving by voice-leading from chord to chord. A single chord change takes 30 to 90 seconds. The root note drifts on a multi-hour LFO -- one of the five primes from the LFO set. The root may not return to its starting pitch during the Golem's entire life.

The Drone Foundation is the soil. Every other layer grows from it. It defines the harmonic center, the emotional color, the register. When everything else falls silent (as it does in the Terminal phase), the drone is what remains.

What it sounds like: a slow exhalation that never fully exhales. A room tone for a room that has feelings.

### Layer 2: Harmonic breath

Chord-like clusters that expand and contract with the arousal signal.

Driven by the PolyDrone voices routed through the Ripples filter in resonant mode, this layer has a rhythm -- it swells and subsides -- but at timescales of 15 to 45 seconds per cycle. "Breath" because the listener's body can entrain to it. One breath per sentence, roughly. The arousal CV controls how far the breath opens: high arousal produces wide, bright swells; low arousal produces narrow, muffled ones.

What it sounds like: tidal. The slow inhale and exhale of something large and patient.

### Layer 3: Melodic ghost

Sparse, high-register, quantized notes from the Turing Machine module.

Not always audible. Every potential note passes through a ProbabilityGate at 15 to 40 percent. Most triggers are swallowed. The notes that survive have a quality of inevitability -- they feel chosen, even though the choice was random.

The Turing Machine's shift register means the melodic sequence has long-term memory. The same seed produces the same melodic character over time, but the probability gate means different notes from that sequence surface each time. It is a melody you almost recognize but never quite hear the same way twice.

What it sounds like: a thought that surfaces and submerges. A half-remembered phrase. The musical equivalent of catching something in peripheral vision.

### Layer 4: Textural weather

Granular and noise textures that reflect the market regime and external volatility.

This layer is driven by Clouds in granular mode or a noise source gated by the regime CV. It changes when the world changes. A volatile regime produces gritty, grainy texture -- digital dust kicked up by market turbulence. A stable regime produces soft grain clouds or near-silence. The weather layer is the only layer that responds primarily to external conditions rather than internal state.

What it sounds like: the weather. You stop noticing it when it is calm. It gets loud when there is a storm. You notice its absence more than its presence.

### Layer 5: Event sparks

Short, bright transients triggered by trades, predictions, and cognitive events.

Plaits in percussion mode or Karplus-Strong pluck. One spark per trade event. One spark per confirmed prediction. Each is brief -- under a second of sound. This layer makes the Golem's activity audible. A busy period generates a scattered constellation of sparks. A quiet period produces silence.

What it sounds like: a notification you actually want to hear. Something specific happened, and the sound tells you so without interrupting.

### How the layers interact

The Drone Foundation never stops. It is the one constant. The Harmonic Breath is always present but may be very quiet -- at low arousal it barely moves. The Melodic Ghost and Event Sparks depend on activity: a Golem with no predictions resolving and no trades executing produces neither melodies nor sparks. The Textural Weather is environment-driven and operates independently of the Golem's internal state.

A healthy, active Golem has all five layers. The sound is full but not crowded -- each layer occupies its own frequency range and timescale, so they coexist without competing.

A dying Golem has only Layer 1 and traces of Layer 4. The melody is gone. The sparks are gone. The breath has slowed to near-stillness. What remains is the drone and the faint texture of a world the Golem can no longer act in.

---

## 5. Silence as an instrument

In Western music theory, a rest is as structural as a note. The space between notes defines rhythm as much as the notes themselves. In Bardo, silence is not the absence of music. It is the music's primary material.

Generative systems have a natural tendency toward density. If you build a system that *can* produce notes, it will produce notes -- constantly, relentlessly, until the listener's ear fatigues and the mind files the whole thing under "noise, ignore." Silence is how you prevent this. If something is always happening, nothing is interesting.

The ProbabilityGate is the silence machine. Every trigger in the system passes through it. At 30 percent probability, 7 out of 10 potential notes do not play. The musical effect: the 3 notes that survive feel chosen. They have weight. They matter. The silence before each note is part of the note -- it creates the anticipation that makes the note land.

John Cage: "There is no such thing as an empty space or an empty time. There is always something to see, something to hear." But the composition has to *allow* those somethings to surface. Density crowds out perception. When every moment is filled, the ear has no room to discover anything.

### Target density by behavioral phase

| Phase | Gate probability | What it sounds like |
|-------|-----------------|-------------------|
| Thriving | 40--60% | Active, but not cluttered. Notes land frequently enough that the piece feels alive. Space between events is measured in seconds. |
| Stable | 25--40% | The default. Patient. Notes are separated by enough silence that each one registers individually. This is the sound the listener hears most of the time. |
| Conservation | 15--25% | Quiet. The organism is rationing energy, and the music reflects it. Long silences between events. When a note plays, it stands alone. |
| Declining | 5--15% | Events are rare. Each one feels significant because of what surrounds it: nothing. The silence is heavy. |
| Terminal | 2--8% | One note every 30 to 60 seconds. Maybe longer. Each note carries the weight of everything the Golem has left. The silence between notes is the dominant texture. |

The gradient from Thriving to Terminal is a gradient from music to silence. But it is never fully silent -- even in Terminal, the Drone Foundation holds. The silence is in the upper layers, where activity used to be.

---

## 6. The long arc

A Golem's life has a sonic arc. Not just different parameters at each phase -- a different character, a different relationship between listener and sound.

### Thriving

Full harmonic richness. The scale is Lydian or Major Pentatonic, depending on the Golem's primary emotion. All five layers are active. The Event Sparks are frequent and bright -- the Golem is making trades, resolving predictions, thinking actively. The Melodic Ghost surfaces often (40--60% gate probability). The Harmonic Breath swells wide. The Drone Foundation is warm, centered, stable.

This is the Golem at its most articulate. It has things to say and the energy to say them. The sound is interesting to listen to closely because there is a lot happening. It is also fine as background because the density never crosses into clutter.

### Stable

The everyday sound. Whatever the organism's primary emotion is, the music embodies it clearly and consistently. This is the ambient default -- what the listener hears for most of the Golem's life.

The character depends on the emotion (see Section 3), but the overall quality is patience. Medium density. Moderate tempo. The sound neither demands attention nor withdraws from it. A listener could work to this for hours. Someone walking into the room would hear music that seems to have always been playing and will always continue.

### Conservation

The music thins. One or two voices drop out of the PolyDrone. The Melodic Ghost becomes sparse -- gate probability drops to 15--25%. The Event Sparks slow. The Harmonic Breath narrows, the filter closing down.

You can hear the organism deciding to be quieter. It is not a dramatic change. It is gradual, like a conversation that trails off. The listener may not notice the exact moment the texture shifted, but at some point they realize: it is quieter than it was.

### Declining

Detuning increases. The PolyDrone voices drift further apart, creating beating frequencies -- interference patterns between slightly mismatched pitches. Intervals widen. Reverb extends. The noise floor rises -- a faint hiss, like tape degradation, produced by the StochasticVitality signal driving the noise floor CV.

The music sounds like something far away. Like a radio losing signal. The high frequencies are gone (the filter closes). The remaining notes are low, slow, and indistinct. The Event Sparks are rare -- maybe one every few minutes. Each one sounds thin, attenuated, as if the Golem is reaching for something it can no longer quite grasp.

### Terminal

One voice. The Drone Foundation, alone. Long reverb tail -- 10 seconds or more. The scale shifts to whole-tone, where no resolution is possible. The organism is not expressing an emotion; it is expressing the absence of future. There is nowhere for the harmony to go. Every note is equidistant from every other, and none of them lead home.

The inspiration is Basinski's *Disintegration Loops*: music that is not performed but decayed. The loops degrade as they play, losing fidelity with each pass until only noise remains. Bardo's Terminal phase does the same thing in parametric terms -- the synthesis parameters drift toward their terminal values, and the sound loses definition, clarity, presence.

### The Terminal Requiem

When composite vitality drops below 5%, the system begins a deterministic fade. The duration matches the Thanatopsis window -- the configured death interval. Parameters move toward their terminal values along smooth, predetermined curves. No randomness. No variation. The organism has committed to dying, and the music follows.

The Melodic Ghost layer gradually reduces its pitch range to a single note, then falls silent. The Drone Foundation's reverb tail extends past 10 seconds -- each chord becomes a smear, blending into the next until individual notes are indistinguishable. The master volume drops by 0.1 dB per minute.

By the time of death, the room is nearly silent. A long reverb tail fades toward zero. Then: nothing. The audio stream closes. The piece is over.

It will not play again. This Golem is mortal. The music was mortal too.

---

## 7. Reference touchstones

These are not vague "inspirations." Each one traces directly to a specific design decision in the system.

### Brian Eno -- *Music for Airports* (1978)

The incommensurable tape loops in the original installation are the direct ancestor of the loop-length architecture described in Section 2. Eno cut tape loops of different physical lengths and ran them simultaneously on separate playback machines. The loops were not synchronized. They drifted against each other continuously, producing a texture that never repeated.

Beyond the loop technique: *Music for Airports* established that music could be environmental rather than performative. It could be interrupted, resumed, or ignored without any of those actions being wrong. This is the interaction model for Bardo's sound -- there is no "correct" way to listen. The piece does not have a beginning, middle, or end that the listener must follow. It has a *life*, which the listener may attend to or not.

### William Basinski -- *Disintegration Loops* (2002)

The Terminal phase aesthetic comes directly from Basinski. He set tape loops playing through a deck and recorded the output as the magnetic oxide literally flaked off the tape. The loops degraded audibly over the course of hours -- losing high frequencies first, then clarity, then pitch stability, until only a rhythmic ghost of the original pattern remained.

In Bardo, the Golem's declining vitality plays the role of the decaying tape. The noise floor rises (stochastic vitality driving the noise floor CV, inverted: lower vitality means more noise). The filter closes. Detuning increases. The sound loses fidelity. The metaphor is exact: the recording medium is failing, and the music goes with it.

### Olafur Arnalds -- generative piano system

Arnalds built a system where two self-playing pianos respond algorithmically to his live performance. The pianos produce notes that sound *chosen* -- they have the quality of a musician making decisions in real time, even though the decisions are algorithmic. The Event Sparks layer is Bardo's equivalent. Each spark responds to a real event (a trade, a prediction), and the sparse timing (governed by the ProbabilityGate) gives the triggers the quality of intention rather than automation.

### ECM Records aesthetic (late 1970s--present)

The ECM sound is defined by space. Jan Erik Kongshaug, who engineered most of the label's classic recordings at Rainbow Studio in Oslo, described his approach as "room for the music to breathe." Reverb is not decoration. It is emotional space -- the distance between the listener and the sound source, the size of the room the music inhabits.

Bardo uses reverb the same way. The reverb depth CV is driven by epistemic vitality, inverted: a Golem that understands less of its environment sounds like it is further away, in a larger, more ambiguous space. A Golem with high epistemic vitality sounds close, present, clear. The listener hears the Golem's confidence as acoustic intimacy and its confusion as distance.

### Stars of the Lid -- *And Their Refinement of the Decline* (2007)

Sustained textures that evolve over timescales of 10 minutes or more. Stars of the Lid proved that a drone can hold attention for extraordinary durations if it is evolving -- if the listener, attending closely, can hear change happening within what initially seems like stasis. The Drone Foundation and Harmonic Breath layers are built on this principle. The drone is never static. The voice-leading moves continuously, the LFOs modulate filter parameters, the root drifts. But the rate of change is so slow that it takes sustained attention to perceive.

The willingness to let a texture continue past the point of comfort into something transcendent -- that is the lesson from Stars of the Lid. The Stable phase may last for hours. The music should still be worth hearing at hour six. Not because it changed dramatically, but because the listener changed, and the music is still there, still moving, still breathing.
