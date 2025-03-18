# Maxwell's demons demo

Demo [itself](https://sitandr.github.io/Maxwell/).

## Traditional Maxwell's demon

> In the thought experiment, a demon controls a small massless door between two chambers of gas. As individual gas molecules (or atoms) approach the door, the demon quickly opens and closes the door to allow only fast-moving molecules to pass through in one direction, and only slow-moving molecules to pass through in the other. Because the kinetic temperature of a gas depends on the velocities of its constituent molecules, the demon's actions cause one chamber to warm up and the other to cool down. This would decrease the total entropy of the system, without applying any work, thereby violating the second law of thermodynamics.
> 
> (Wiki)

Imagine a tiny device that allows fast-moving molecules pass through one direction, and one to other. Sounds simple, it's not too hard to design such things for, say, billiard-balls. But physics say that such a device would violate on of the most fundamental physics law — one chamber will heat, second will cool. What is wrong?

There are several approaches to explain this phenomena, the most common approach says something like "Well, for this device to work it needs to measure speed of molecules or to calculate the speeds using some unreversible operations. In both cases it will cost more energy that is produces, either for the measurements themselves or to erase the memory to free his inner digital space for futher calculations".

It's a cool way to understand the general idea, but there is another interesting way to comprehend "the illegality" of such thing. 

Let's start from considering a much simplier demon that works roughly in the same way.

## One-way-valve ("diode" demon)

Instead of bothering about molecules' speeds, let's just let them pass only one way, opening "the door" for those coming from the left, but not from the right. So most of the molecules will soon move to the right. Now we can just make a small hole between the chambers and install a small "windmill" there, so we can now effectively convert energy of brownian motion into work… Sounds tricky, doesn't it? 

Well, such system seriously violates such general physics' idea as *reversability*. Laws of motion of our molecules (we can consider them as simple "balls" for simplicity) "don't know" anything about the direction of time flow. We can reverse the time, let's just reverse the speeds of all the balls and we will get the same system evolution, but reversed in time.

The problem of our demon is that this idea suddenly stops working. We *can't reverse the time*.

Indeed, when we see a ball coming *from the demon*, we can't say for sure were it came from. It could be *reflected from the same chamber* it is now, or it could *pass the demon from the other side*. We don't know.

That doesn't necessarily implies that such demon is impossible, but it says it can't be "something simple" — "simple things" obey the reversability principle. It may be possible to create such a device using, e.g., an external system that detects nearby molecules and opens/closes the door with electric impulses. Obviously, such a system needs much more energy it can produce.

However, this is not the thing this demo was created for. There is an interesting "hack" that violates reversibilty in a much more obscure way. So, let's consider the following demon…

## "Tennis" demon.

If it is that important for "demons" to be reversable, so we can reconstruct the trajectories of the balls that have collided with them, it may be a good idea to create a bijective function between speeds "before" and "after". That means there should be always the one same way that speed from one side converts to the speed on the other and back.

Obviously, the simpliest examples of such functions are "always pass without changes" (empty filter/device, just a "hole" in wall) and "always reflect at the same angle" (no hole, just a wall). These are not interesting to investigate.

One more interesing example of such function is a "tennis demon" suggested by P. A. Skordos in [Compressible dynamics, time reversibility, Maxwell’s demon, and the second law, 1992]. The idea is that demon "places" his rocket in different positions depending on the angle and side the ball is coming from. 

(there will be an image)

And yeah, it is bijective.

But if you launch this demo and discover that this demon *actually creates the density difference between the chambers*. So it violates the Second Law without violating reversability. As you may suspect, that's not the point — the demon violates the reversibility *in some other way*.

It is useful to consider so-called "phase-space". The idea is exploring "the density" of all possible "coordinate&speed" combinations and how does it change when a ball passes through the demon. I would skip the calculations — the answer it that for the truthly reversible (volume-saving) demons the Jacobian in space of possible coords&speeds&angles should be equal to 1 (if you are interested, I recommend to read the article). It is not that easy to understand, but it is quite a beutiful idea.

Tennis demon saves the angle, but not the speeds. That actually means that there are more balls moving perpendicular to the demon's plane than parallel ones. So there will be more balls coming from left to right, than from right to left.

Using the theory of phase-conserving transformations, we can easily find the "good" class of demons…

## Phase-conserving demons

It is easy to show that such functions will look like (again, for the calculations see the article):

`sin⁻¹(± sin θ + C)`, where `C` is some constant. If `C = 0`, it easy to see that it describes either a "hole" or a "wall".

You can select "phase-conserving" filter in demo to see that there is no significant chamber density difference. Again, it can be analytically proofed, but I skip it there (see the article).

It is important to note that this function describes only determined speed-to-speed conversion functions. There may be other solutions, based on dependence of the coordinates or some probalities.

## Limitations

The demo is intended to show the work of demons, it doesn't have a goal to create an ideal simulation. There may be problems with large numbers of large balls colliding into each other and similar. Also it is important to note that *most of the parameters change only after pressing "restart"* to prevent some undesirible "runtime" strange cases.

I just wanted to write it quickly, effectively and web-available.

Rust library egui does it almost perfectly. Something harder may be problematical.
