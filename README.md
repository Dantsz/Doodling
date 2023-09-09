# Doodling
Web-app for quickly producing and sharing doodles.
## Why?
I wanted to try out wgpu on browser and I wanted to try out htmx. I decided to use as much unfamiliar tech as possible to make it more interesting, so I went with surrealDB for storing the doodles.
## Installation
### Docker compose install
This should be the easiest way to get the app up and running.
1. clone the repo
2. run `docker-compose up`
### Development install
There's some extra manual steps needed, to which I didn't find a good enough solution fast enough:
1. The first is to setup the tailwind CLI to watch for changes as describe in the DoodleHtmx readme.
2. The second is to manually compile and put the build of the doodling widget in the resources folder.(the dockerfile does is an example of how to do it)

## TODO
* [ ] Add a page to view a specific doodle and option to get that link when creating a doodle
* [ ] Build wasm inside docker image
* [ ] Improve the drawing widget (make the painting more consistenten(as opossed to just drawing a black square to the screen when the use clicks the canvas),add colors, eraser, etc.)