// TODO
/*
Node types to implement:
- Sprites: show character, hide sprite (with emotions)
- Change Background
- Tweens
- define user variables
- IF routes
*/

let meta = JSON.parse(metadataString)

document.getElementById("T").innerHTML = meta.title

let messageBox = undefined,
bgSprite = undefined,
choices = undefined,
sprites = undefined,
message,
speaker,
choicesBg = undefined

let activeSprites = new Object()
let spriteProperties = new Object()
let effects = new Object()
let lock = false

let isMessageBoxInteractive = true

let scene
let currentNode
let currentDialogueLine = 0
let tapCountDown = 0

let windowWidth = meta.width
let windowHeight = meta.height

let music = undefined
let musicID = undefined

let g = hexi(meta.width, meta.height, setup, thingsToLoad, load)
g.scaleToWindow()
g.start()
console.log(g)

let story = []


function load(){
	g.loadingBar()
}

function setup() {
    g.fps = 30;
	//setScene(g.json("scenes/main.json"))
	constructMenu()
	//currentNode = scene[0]
    //g.border = "1px solid black"
}

function play(){
	let hit = detectHitRecursive(g.stage.children)
	// hit == true ? console.log("hit") : console.log("no hit")
}

function reset() {
	messageBox = undefined
	bgSprite = undefined
	choices = undefined
	sprites = undefined
	message = undefined
	speaker = undefined
	choicesBg = undefined

	activeSprites = new Object()
	spriteProperties = new Object()
	effects = new Object()

	isMessageBoxInteractive = true

	currentDialogueLine = 0
	tapCountDown = 0

	//music = undefined
	//musicID = undefined
}

function constructMenu() {
	if (music && music.playing == true) {
		music.pause()
	}
	g.stage.children.forEach(child => {
		child.interact = false
		child.tap = undefined
	})
	g.stage.removeChildren(0, undefined)
	let rect = g.rectangle(windowWidth, windowHeight, "dark grey")
	rect.layer = -2

	let title = g.text(meta.title, "" + meta.titleSize + "px garamond", "white");

	let begin = g.text("Start", "" + meta.titleSize / 1.5 + "px garamond", "white")
	let cont = g.text("Continue", "" + meta.titleSize / 1.5 + "px garamond", "white")
	begin.interact = true
	begin.tap = () => {
		reset()
		story = []
		console.log(g)
		begin.interact = false
		cont.interact = false
		setScene(g.json("scenes/main.json"), "scenes/main.json")
		resolveCurrentNode()
	}

	cont.interact = true
	cont.tap = () => {
		reset()
		begin.interact = false
		cont.interact = false
		let ss = localStorage.getItem("save"+meta.title+"svstd")
		if (ss) {
			ss = JSON.parse(ss)
			let cur = ss.story.pop()
			setScene(g.json("scenes/main.json"))
			ss.story.forEach(node => {
				applyNode(scene[node])
			})

			currentNode = scene[cur]
			resolveCurrentNode()
		}
		else {
			setScene(g.json("scenes/main.json"), "scenes/main.json")
			resolveCurrentNode()
		}
	}

	g.stage.putCenter(title, 0, meta.titleSize * -2)
	let line = g.rectangle(title.halfWidth * 2, 2)
	line.x = title.x
	line.y = title.y + meta.titleSize * 1.5

	g.stage.putCenter(begin)
	g.stage.putCenter(cont, 0, begin.halfHeight * 3)
}

let saveRect = undefined
let menuRect = undefined

function setScene(newScene, p) {
	if (music && music.playing == true) {
		music.pause()
	}
	scene = newScene
	g.stage.removeChildren(0, undefined)

	let saveButton = g.text("Save", "" + meta.fontSize + "px " + meta.choiceFontName, "white")
	saveRect = g.rectangle(saveButton.halfWidth * 2.5, saveButton.halfHeight * 2.5, "black")
	saveRect.alpha = 0.9
	saveRect.layer = 128
	saveButton.layer = 128
	saveRect.addChild(saveButton)
	saveRect.putCenter(saveButton)
	saveButton.interact = true
	saveButton.tap = () => {
		let ss = new Object()
		ss.story = story

		//console.log(ss)
		localStorage.setItem("save"+meta.title+"svstd", JSON.stringify(ss))
	}

	let menuButton = g.text("Exit", "" + meta.fontSize + "px "+ meta.choiceFontName, "white")
	menuRect = g.rectangle(menuButton.halfWidth * 2.5, menuButton.halfHeight * 2.5, "black")
	menuRect.alpha = 0.9
	menuRect.layer = 128
	menuButton.layer = 128
	menuRect.addChild(menuButton)
	menuRect.putCenter(menuButton)
	menuButton.interact = true
	menuButton.tap = () => {
		constructMenu()
	}
	menuRect.x += meta.width - menuRect.halfWidth * 2
	saveRect.x += meta.width - menuRect.halfWidth * 2 - saveRect.halfWidth * 2 - meta.choiceFontSize

	messageBox = g.rectangle(windowWidth, windowHeight / 5, 
		rgbToHex(meta.msgBoxColor[0], meta.msgBoxColor[1], meta.msgBoxColor[2],))
    messageBox.alpha = meta.msgBoxOpacity / 100;
	messageBox.interact = true
	// g.makeInteractive(messageBox);
	isMessageBoxInteractive = true
	messageBox.tap = onMesssageBoxTap;
	messageBox.layer = 33
	g.stage.putBottom(messageBox, 0, -windowHeight / 5)

    message = g.text("Message", "" + meta.fontSize + "px " + meta.fontName, rgbToHex(meta.fontColor[0], meta.fontColor[1], meta.fontColor[2]))
	message.layer = 34
    g.stage.putBottom(message, 0, -windowHeight / 5)

	speaker = g.text("Speaker", "" + meta.speakerFontSize + "px " + meta.speakerFontName, rgbToHex(meta.speakerFontColor[0], meta.speakerFontColor[1], meta.speakerFontColor[2]))
	speaker.layer = 35
	g.stage.putBottom(speaker, 0, -windowHeight / 5 + 8)
	speaker.position.x = 128
	
	choices = g.group()
	choices.layer = 35

	g.pointer.tap = handleTapGeneral

	// console.log(g.text)
	// console.log(g)

	g.state = play

	currentNode = scene[0]
}
function setCurrentNode(newNode) {
	if (!newNode) {
		constructMenu()
		return
	}

	currentNode = newNode
	currentDialogueLine = 0
	tapCountDown = 0
	story.push(currentNode.id.id)
	resolveCurrentNode()
}

function handleTapGeneral() {
	if (lock == true) return
	if (g.hit(g.pointer, saveRect) == true
		|| g.hit(g.pointer, menuRect) == true) {
			return
		}

	let hit = detectHitRecursive(g.stage.children)
	hit = false
	// hit == true ? console.log("hit") : console.log("no hit")
	if (hit == false && currentNode.type.text == "dialogue") {
		onMesssageBoxTap()
		return
	}
	if (hit == false && currentNode.type.text == "wait") {
		tapCountDown += 1
		if (tapCountDown >= 1) setCurrentNode(scene[currentNode.next.next])
		return
	}
	if (hit == false && currentNode.next.next) {
		setCurrentNode(scene[currentNode.next.next])
	}
}

function detectHitRecursive(that) {
	let nextIter = []
	let result = false
	that.forEach(child => {
		if (g.hit(g.pointer, child) == true) {
			// console.log(child)
			result = true
			if (child === bgSprite) result = false;
		}
		if (child.children.length > 0) nextIter.concat(child.children)
	})
	if (result == true) return result
	if (nextIter.length == 0) return false
	
	return detectHitRecursive(nextIter)
}

function toggleMessageBox(visibility) {
	if (visibility == false) {
		// console.log("box hidden")
		messageBox.interact = false
		messageBox.tap = () => {}
		g.stage.removeChild(messageBox)
		g.stage.removeChild(message)
		g.stage.removeChild(speaker)
	}
	else {
		if (g.stage.children.includes(messageBox) == false) g.stage.addChild(messageBox)
		if (g.stage.children.includes(message) == false) g.stage.addChild(message)
		if (g.stage.children.includes(speaker) == false) g.stage.addChild(speaker)
	}
}

function applyNode(node) {
	messageBox.interact = false
	switch (node.type.text) {
		case "root":
			toggleMessageBox(false)
			break;
		case "clear":
			for (char in activeSprites) {
				activeSprites[char].texture = g.tink.Texture.EMPTY
			}
			break
		case "dialogue":
			toggleMessageBox(true)

			messageBox.interact = true
			message.content = node.text.dialogue[node.text.dialogue.length - 1];

			g.stage.putBottom(message, 0, -windowHeight / 6.5)

			if (node.speaker.text) {
				speaker.content = node.speaker.text + ":"
			}
			else {
				speaker.content = ""
			}
			g.stage.putBottom(speaker, 0, -windowHeight / 5 + 8)
			speaker.position.x = 256
			break;
		case "hide messagebox":
			toggleMessageBox(false)
			break;
		case "show character":
			let spritePath = "assets/characters/"
			if (node.expression.text) spritePath += node.sprite.text + "_" + node.expression.text + node.extension.text
			else spritePath += node.sprite.text + "" + node.extension.text

			//g.stage.removeChild(activeSprites[node.sprite.text])
			//activeSprites[node.sprite.text] = g.sprite(spritePath)
			if (Object.hasOwn(activeSprites, node.sprite.text)) {
				let sp = activeSprites[node.sprite.text]
				sp.texture = g.tink.Texture.fromImage(spritePath)
			}
			else {
				activeSprites[node.sprite.text] = g.sprite(spritePath)
			}
			if (!spriteProperties[node.sprite.text]) spriteProperties[node.sprite.text] = new Object()

			if (node.hasOwnProperty("properties")) {
				for (let prop in node.properties.properties) {
					spriteProperties[node.sprite.text][prop] = node.properties.properties[prop]
				}
			}

			let thisSprite = activeSprites[node.sprite.text]
			thisSprite.pivotX = 0.5
			thisSprite.pivotY = 1

			let thisProperties = spriteProperties[node.sprite.text]
			// console.log(thisProperties)

				thisProperties.x == null ? {} : thisSprite.x = thisProperties.x * windowWidth
				thisProperties.y == null ? thisSprite.y = windowHeight : thisSprite.y = thisProperties.y * windowHeight
				thisProperties.scaleX == null ? {} : thisSprite.scaleX = thisProperties.scaleX / 100
				thisProperties.scaleY == null ? {} : thisSprite.scaleY = thisProperties.scaleY / 100
				thisProperties.scale == null ? {} : (thisSprite.scaleX = thisProperties.scale / 100,
					thisSprite.scaleY = thisProperties.scale),
				thisProperties.layer == null ? {} : thisSprite.layer = thisProperties.layer
			
			
			thisSprite.interact = true
			//thisSprite.layer = 1
			break
		case "hide character":
			activeSprites[node.sprite.text].texture = g.tink.Texture.EMPTY
			//g.stage.removeChild(activeSprites[node.sprite.text])
			break
		case "set background":
			let bgPath = "assets/backgrounds/" + node.sprite.text + node.extension.text
			if (bgSprite) g.stage.removeChild(bgSprite)
			bgSprite = g.sprite(bgPath)
			bgSprite.width = windowWidth
			bgSprite.height = windowHeight
			bgSprite.layer = -1
			break
		case "set background color":
			if (bgSprite) g.stage.removeChild(bgSprite)
			bgSprite = g.rectangle(
				windowWidth,
				windowHeight,
				rgbToHex(
					node.color.rgb[0],
					node.color.rgb[1],
					node.color.rgb[2]
				)
			)
			bgSprite.layer = -1
			break
		case "play music":
			let musicPath = "assets/music/" + node.sfx.text + node.extension.text
			if (musicID == musicPath) {
				music.play()
			}
			else {
				musicID = musicPath
				music = g.sound(musicPath)
				music.volume = node.volume.float
				music.loop = true
				music.play()
			}
			break
		case "stop music":
			if (music && music.playing == true) {
				music.pause()
			}
			break
		case "switch scene":
			let p = "scenes/" + node.scene.text + ".json"
			reset()
			setScene(g.json(p), p)
			break
		case "breathe":
			if (node.sprite.text) {
				let sprite = activeSprites[node.sprite.text]
				let prop = node.properties.properties
				let yo = false
				if (prop.Repeat == 1) { yo = true }
				let vfx = g.breathe(
					sprite,
					prop["Final X"],
					prop["Final Y"],
					prop.Duration * 30,
					yo
				)
				if (Array.isArray(effects[node.sprite.text])) {
					effects[node.sprite.text].push(vfx)
				}
				else {
					effects[node.sprite.text] = []
					effects[node.sprite.text].push(vfx)
				}
			}
			break
		case "slide":
			if (node.sprite.text) {
				let sprite = activeSprites[node.sprite.text]
				let prop = node.properties.properties
				let yo = false
				if (prop.Repeat == 1) { yo = true }
				let vfx = g.slide(
					sprite,
					prop["Target X"] * windowWidth,
					prop["Target Y"] * windowHeight,
					prop.Duration * 30,
					undefined,
					yo
				)
				if (Array.isArray(effects[node.sprite.text])) {
					effects[node.sprite.text].push(vfx)
				}
				else {
					effects[node.sprite.text] = []
					effects[node.sprite.text].push(vfx)
				}
				let props = spriteProperties[node.sprite.text]
				props.x = prop["Target X"]
				props.y = prop["Target Y"]
			}
			break
		case "clear effects":
			if (node.sprite.text) {
				let arr = effects[node.sprite.text]
				arr.forEach(vfx => {
					vfx.completed = (vfx) => {
						vfx.pause()
					}
					vfx.onComplete = (vfx) => {
						vfx.pause()
					}
				})
			}
			break
		case "pulse":
			if (node.sprite.text) {
				let sprite = activeSprites[node.sprite.text]
				let prop = node.properties.properties
				let yo = false
				let props = spriteProperties[node.sprite.text]
				if (prop.Repeat == 1) { yo = true }
				let vfx = g.pulse(
					sprite,
					prop.Duration * 30,
					prop.Alpha
				)
				if (yo == false) {
					vfx.onComplete = () => { vfx.pause() }
				}
				if (Array.isArray(effects[node.sprite.text])) {
					effects[node.sprite.text].push(vfx)
				}
				else {
					effects[node.sprite.text] = []
					effects[node.sprite.text].push(vfx)
				}
			}
			break
	}
}

function resolveCurrentNode() {
	// console.log(currentNode)
	messageBox.interact = false
	switch (currentNode.type.text) {
		case "root":
			toggleMessageBox(false)
			setCurrentNode(scene[currentNode.next.next])
			break;
		case "dialogue":
			// isMessageBoxInteractive = true
			toggleMessageBox(true)

			messageBox.interact = true
			message.content = currentNode.text.dialogue[currentDialogueLine];

			g.stage.putBottom(message, 0, -windowHeight / 6.5)

			if (currentNode.speaker.text) {
				speaker.content = currentNode.speaker.text + ":"
			}
			else {
				speaker.content = ""
			}
			g.stage.putBottom(speaker, 0, -windowHeight / 5 + 8)
			speaker.position.x = 256
			break;
		case "clear":
			//console.log(activeSprites)
			for (char in activeSprites) {
				//g.stage.removeChild[activeSprites[char]]
				activeSprites[char].texture = g.tink.Texture.EMPTY
			}
			setCurrentNode(scene[currentNode.next.next])
			break
		case "choice":
			for (choice in currentNode.choices.choice) {
				let ctext = g.text("|>  " + currentNode.choices.choice[choice] + "  <|",
					"" + meta.choiceFontSize + "px " + meta.choiceFontName,
					rgbToHex(meta.choiceFontColor[0], meta.choiceFontColor[1], meta.choiceFontColor[2]))
				choices.addChild(ctext)
				ctext.interact = true
				//g.makeInteractive(ctext)
				ctext.tap = onChoiceTap(currentNode.destinations.destination[choice])
				//console.log(ctext)
			}

			// g.stage.putCenter(choices.children[0])
    		// g.flowDown(10, choices.children)
			for (let i = 1; i < choices.children.length; i += 1) {
				choices.children[i].y = choices.children[i - 1].y + choices.children[i - 1].halfHeight * 2 + 32
			}
			choicesBg = g.rectangle(choices.halfWidth * 4 + 64, choices.halfHeight * 2 + 64,
				rgbToHex(meta.choiceBg[0], meta.choiceBg[1], meta.choiceBg[2])
			)
			choicesBg.alpha = meta.choiceOpacity / 100
			choicesBg.layer = 34
			g.stage.putCenter(choicesBg, 0, -choicesBg.halfHeight + 32)
			g.stage.putCenter(choices, 0, -choices.halfHeight)

			break;
		case "hide messagebox":
			toggleMessageBox(false)
			setCurrentNode(scene[currentNode.next.next])
			break;
		case "show character":
			let spritePath = "assets/characters/"
			if (currentNode.expression.text) spritePath += currentNode.sprite.text + "_" + currentNode.expression.text + currentNode.extension.text
			else spritePath += currentNode.sprite.text + "" + currentNode.extension.text

			//g.stage.removeChild(activeSprites[currentNode.sprite.text])
			if (Object.hasOwn(activeSprites, currentNode.sprite.text)) {
				let sp = activeSprites[currentNode.sprite.text]
				sp.texture = g.tink.Texture.fromImage(spritePath)
			}
			else {
				activeSprites[currentNode.sprite.text] = g.sprite(spritePath)
			}
			//activeSprites[currentNode.sprite.text] = g.sprite(spritePath)
			if (!spriteProperties[currentNode.sprite.text]) spriteProperties[currentNode.sprite.text] = new Object()

			if (currentNode.hasOwnProperty("properties")) {
				for (let prop in currentNode.properties.properties) {
					spriteProperties[currentNode.sprite.text][prop] = currentNode.properties.properties[prop]
				}
			}

			let thisSprite = activeSprites[currentNode.sprite.text]
			thisSprite.pivotX = 0.5
			thisSprite.pivotY = 1

			let thisProperties = spriteProperties[currentNode.sprite.text]
			// console.log(thisProperties)

				thisProperties.x == null ? {} : thisSprite.x = thisProperties.x * windowWidth
				thisProperties.y == null ? thisSprite.y = windowHeight : thisSprite.y = thisProperties.y * windowHeight
				thisProperties.scaleX == null ? {} : thisSprite.scaleX = thisProperties.scaleX / 100
				thisProperties.scaleY == null ? {} : thisSprite.scaleY = thisProperties.scaleY / 100
				thisProperties.scale == null ? {} : (thisSprite.scaleX = thisProperties.scale / 100,
					thisSprite.scaleY = thisProperties.scale),
				thisProperties.layer == null ? thisSprite.layer = 1 : thisSprite.layer = thisProperties.layer
				thisProperties.opacity == null ? {} : thisSprite.alpha = thisProperties.opacity / 100
			
			thisSprite.interact = false
			//thisSprite.layer = 1
			console.log(thisSprite)
			setCurrentNode(scene[currentNode.next.next])
			break
		case "hide character":
			console.log(currentNode)
			activeSprites[currentNode.sprite.text].texture = g.tink.Texture.EMPTY
			//g.stage.removeChild(activeSprites[currentNode.sprite.text])
			setCurrentNode(scene[currentNode.next.next])
			break
		case "set background":
			let bgPath = "assets/backgrounds/" + currentNode.sprite.text + currentNode.extension.text
			if (bgSprite) g.stage.removeChild(bgSprite)
			bgSprite = g.sprite(bgPath)
			bgSprite.width = windowWidth
			bgSprite.height = windowHeight
			bgSprite.layer = -1
			setCurrentNode(scene[currentNode.next.next])
			break
		case "set background color":
			if (bgSprite) g.stage.removeChild(bgSprite)
			bgSprite = g.rectangle(
				windowWidth,
				windowHeight,
				rgbToHex(
					currentNode.color.rgb[0],
					currentNode.color.rgb[1],
					currentNode.color.rgb[2]
				)
			)
			bgSprite.layer = -1
			setCurrentNode(scene[currentNode.next.next])
			break
		case "wait":
			break
		case "wait for":
			lock = true
			setTimeout(() => {
				lock = false
				setCurrentNode(scene[currentNode.next.next])
				resolveCurrentNode()
			}, currentNode.timeout.float * 1000);
			break
		case "play music":
			let musicPath = "assets/music/" + currentNode.sfx.text + currentNode.extension.text
			if (musicID == musicPath) {
				music.play()
			}
			else {
				musicID = musicPath
				music = g.sound(musicPath)
				music.volume = currentNode.volume.float
				music.loop = true
				music.play()
			}
			setCurrentNode(scene[currentNode.next.next])
			break
		case "stop music":
			if (music && music.playing == true) {
				music.pause()
			}
			setCurrentNode(scene[currentNode.next.next])
			break
		case "play sfx":
			let sfxPath = "assets/sfx/" + currentNode.sfx.text + currentNode.extension.text
			let sfx = g.sound(sfxPath)
			sfx.volume = currentNode.volume.float
			sfx.play()
			console.log(sfx)
			setCurrentNode(scene[currentNode.next.next])
		case "switch scene":
			let p = "scenes/" + currentNode.scene.text + ".json"
			reset()
			setScene(g.json(p), p)
			resolveCurrentNode()
			break
		case "breathe":
			if (currentNode.sprite.text) {
				let sprite = activeSprites[currentNode.sprite.text]
				let prop = currentNode.properties.properties
				let yo = false
				if (prop.Repeat == 1) { yo = true }
				let vfx = g.breathe(
					sprite,
					prop["Final X"],
					prop["Final Y"],
					prop.Duration * 30,
					yo
				)
				if (Array.isArray(effects[currentNode.sprite.text])) {
					effects[currentNode.sprite.text].push(vfx)
				}
				else {
					effects[currentNode.sprite.text] = []
					effects[currentNode.sprite.text].push(vfx)
				}
			}
			setCurrentNode(scene[currentNode.next.next])
			break
		case "slide":
			if (currentNode.sprite.text) {
				let sprite = activeSprites[currentNode.sprite.text]
				let prop = currentNode.properties.properties
				let yo = false
				if (prop.Repeat == 1) { yo = true }
				let vfx = g.slide(
					sprite,
					prop["Target X"] * windowWidth,
					prop["Target Y"] * windowHeight,
					prop.Duration * 30,
					undefined,
					yo
				)
				if (Array.isArray(effects[currentNode.sprite.text])) {
					effects[currentNode.sprite.text].push(vfx)
				}
				else {
					effects[currentNode.sprite.text] = []
					effects[currentNode.sprite.text].push(vfx)
				}
				let props = spriteProperties[currentNode.sprite.text]
				props.x = prop["Target X"]
				props.y = prop["Target Y"]
			}
			setCurrentNode(scene[currentNode.next.next])
			break
		case "clear effects":
			if (currentNode.sprite.text) {
				let arr = effects[currentNode.sprite.text]
				arr.forEach(vfx => {
					vfx.completed = (vfx) => {
						vfx.pause()
					}
					vfx.onComplete = (vfx) => {
						vfx.pause()
					}
				})
			}
			setCurrentNode(scene[currentNode.next.next])
			break
		case "pulse":
			if (currentNode.sprite.text) {
				let sprite = activeSprites[currentNode.sprite.text]
				let prop = currentNode.properties.properties
				let yo = false
				let props = spriteProperties[currentNode.sprite.text]
				if (prop.Repeat == 1) { yo = true }
				let vfx = g.pulse(
					sprite,
					prop.Duration * 30,
					prop.Alpha
				)
				if (yo == false) {
					vfx.onComplete = () => { vfx.pause() }
				}
				if (Array.isArray(effects[currentNode.sprite.text])) {
					effects[currentNode.sprite.text].push(vfx)
				}
				else {
					effects[currentNode.sprite.text] = []
					effects[currentNode.sprite.text].push(vfx)
				}
			}
			setCurrentNode(scene[currentNode.next.next])
			break
	}
}

function componentToHex(c) {
	c = Math.round(c * 255)
	if (c < 0) {
		c = 0;
	}
	if (c > 255) {
		c = 255
	}
	var hex = c.toString(16);
	return hex.length == 1 ? "0" + hex : hex;
}

function rgbToHex(r, g, b) {
	return "#" + componentToHex(r) + componentToHex(g) + componentToHex(b);
}

function onChoiceTap(choiceID) {
	return function() {
		console.log("Choice clicked!")
		console.log(choiceID)
		// choices = g.group()
		choices.children.forEach(child => {
			child.tap = () => {}
			child.interact = false
		})
		choices.x = windowWidth * 2
		choices.y = windowHeight * 2
		choices.removeChildren(0, undefined)
		choicesBg.alpha = 0
		g.stage.removeChild(choicesBg)
		setCurrentNode(scene[choiceID])
	}
}

function onMesssageBoxTap() {
	// console.log("message box tapped")
	if (isMessageBoxInteractive == false) return;
	
	let length = currentNode.text.dialogue.length
	if (currentDialogueLine < length - 1) {
		currentDialogueLine += 1
		resolveCurrentNode()
	}
	else {
		if (!currentNode.next.next) return;
		messageBox.interact = false
		setCurrentNode(scene[currentNode.next.next])
	}
}