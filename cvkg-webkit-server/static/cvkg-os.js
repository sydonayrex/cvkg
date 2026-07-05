// Lightning Cursor Effect
const LightningCursor = {
    canvas: null,
    ctx: null,
    mouse: { x: -100, y: -100 },
    bolts: [],

    init() {
        this.canvas = document.createElement('canvas');
        this.canvas.style.position = 'fixed';
        this.canvas.style.top = '0';
        this.canvas.style.left = '0';
        this.canvas.style.pointerEvents = 'none';
        this.canvas.style.zIndex = '10001';
        document.body.appendChild(this.canvas);
        this.ctx = this.canvas.getContext('2d');

        window.addEventListener('resize', () => this.resize());
        window.addEventListener('mousemove', (e) => {
            this.mouse.x = e.clientX;
            this.mouse.y = e.clientY;
            if (Math.random() > 0.7) this.createBolt();
        });

        this.resize();
        this.animate();
    },

    resize() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
    },

    createBolt() {
        this.bolts.push({
            x: this.mouse.x,
            y: this.mouse.y,
            life: 1.0,
            segments: this.generateSegments(this.mouse.x, this.mouse.y)
        });
    },

    generateSegments(x, y) {
        const segments = [];
        let curX = x, curY = y;
        for (let i = 0; i < 5; i++) {
            const nx = curX + (Math.random() - 0.5) * 30;
            const ny = curY + (Math.random() - 0.5) * 30;
            segments.push({ x1: curX, y1: curY, x2: nx, y2: ny });
            curX = nx; curY = ny;
        }
        return segments;
    },

    animate() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.strokeStyle = '#00ffff';
        this.ctx.shadowBlur = 10;
        this.ctx.shadowColor = '#00ffff';

        for (let i = this.bolts.length - 1; i >= 0; i--) {
            const b = this.bolts[i];
            b.life -= 0.1;
            if (b.life <= 0) {
                this.bolts.splice(i, 1);
                continue;
            }

            this.ctx.globalAlpha = b.life;
            this.ctx.lineWidth = 2 * b.life;
            this.ctx.beginPath();
            b.segments.forEach(s => {
                this.ctx.moveTo(s.x1, s.y1);
                this.ctx.lineTo(s.x2, s.y2);
            });
            this.ctx.stroke();
        }

        requestAnimationFrame(() => this.animate());
    }
};
LightningCursor.init();

// Lightning System - Subtle
const Lightning = {
    canvas: document.getElementById('lightning-canvas'),
    ctx: null,
    init() {
        this.ctx = this.canvas.getContext('2d');
        window.addEventListener('resize', () => {
            this.canvas.width = window.innerWidth;
            this.canvas.height = window.innerHeight;
        });
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
        this.loop();
    },
    drawBolt(x, y, len, angle, branches) {
        if (branches <= 0 || len <= 0) return;
        const endX = x + Math.cos(angle) * len;
        const endY = y + Math.sin(angle) * len;
        this.ctx.beginPath();
        this.ctx.moveTo(x, y);
        this.ctx.lineTo(endX, endY);
        this.ctx.stroke();
        if (Math.random() > 0.8) this.drawBolt(endX, endY, len * 0.7, angle + 0.6, branches - 1);
        this.drawBolt(endX, endY, len * 0.9, angle + (Math.random() - 0.5) * 0.4, branches - 1);
    },
    loop() {
        if (Math.random() > 0.99) {
            this.ctx.strokeStyle = 'rgba(0, 255, 255, 0.3)';
            this.ctx.lineWidth = 1;
            this.drawBolt(Math.random() * this.canvas.width, 0, 40, Math.PI/2, 15);
            setTimeout(() => this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height), 60);
        }
        setTimeout(() => this.loop(), 100);
    }
};
Lightning.init();

// Mjolnir Shatter Logic
const Mjolnir = {
    shatter(x, y) {
        for (let i = 0; i < 20; i++) {
            const f = document.createElement('div');
            f.className = 'shatter-fragment';
            f.style.left = x + 'px'; f.style.top = y + 'px';
            document.body.appendChild(f);
            const a = Math.random() * Math.PI * 2;
            const s = Math.random() * 8 + 4;
            const vx = Math.cos(a) * s;
            const vy = Math.sin(a) * s;
            let o = 1;
            const step = () => {
                f.style.left = (parseFloat(f.style.left) + vx) + 'px';
                f.style.top = (parseFloat(f.style.top) + vy) + 'px';
                o -= 0.03; f.style.opacity = o;
                if (o > 0) requestAnimationFrame(step); else f.remove();
            };
            requestAnimationFrame(step);
        }
        document.body.style.transform = `translate(${(Math.random()-0.5)*12}px, ${(Math.random()-0.5)*12}px)`;
        setTimeout(() => document.body.style.transform = '', 100);
    }
};

// Safe HTML Escaper Helper to prevent XSS (H2)
function escapeHTML(str) {
    if (!str) return '';
    return str.replace(/[&<>'"]/g, 
        tag => ({
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            "'": '&#39;',
            '"': '&quot;'
        }[tag] || tag)
    );
}

// CVKG OS Core System
const CVKG = {
    windows: {},
    zIndex: 100,
    fs: {
        '/': { type: 'dir', children: ['Documents', 'Images'] },
        '/Documents': { type: 'dir', children: ['Viking_Code.rs'] },
        '/Images': { type: 'dir', children: [] },
        '/Documents/Viking_Code.rs': { type: 'file', content: 'pub fn berserk() { ... }' }
    },
    currentPath: '/',

    init() {
        this.createDesktopIcons();
        this.updateClock();
        setInterval(() => this.updateClock(), 1000);
        this.setupEventListeners();
    },

    createDesktopIcons() {
        const icons = [
            { name: 'System Root', action: 'filemanager', icon: 'folder' },
            { name: 'Viking Saga', action: 'vikinggta', icon: 'shield' },
            { name: 'Terminal', action: 'terminal', icon: 'terminal' }
        ];

        icons.forEach((icon, i) => {
            const el = document.createElement('div');
            el.className = 'desktop-icon';
            el.style.left = '40px';
            el.style.top = (40 + i * 120) + 'px';

            const iconImg = document.createElement('div');
            iconImg.className = 'icon-img';
            
            if (icon.icon === 'folder') {
                iconImg.innerHTML = '<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>';
            } else if (icon.icon === 'shield') {
                iconImg.innerHTML = '<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="3"/><path d="M12 2v20M2 12h20"/></svg>';
            } else {
                iconImg.innerHTML = '<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>';
            }

            const iconName = document.createElement('div');
            iconName.className = 'icon-name';
            iconName.textContent = icon.name;

            el.appendChild(iconImg);
            el.appendChild(iconName);
            
            el.dataset.action = icon.action;
            el.addEventListener('dblclick', (e) => {
                Mjolnir.shatter(e.clientX, e.clientY);
                this.openApp(icon.action);
            });
            document.getElementById('desktop').appendChild(el);
        });
    },

    toggleStartMenu() {
        document.getElementById('start-menu').classList.toggle('active');
    },

    openApp(name) {
        if (!this.windows[name]) {
            this.createWindow(name);
        } else {
            this.focusWindow(name);
        }
        document.getElementById('start-menu').classList.remove('active');
    },

    createWindow(name) {
        const win = document.createElement('div');
        win.className = 'window';
        win.style.left = (150 + Object.keys(this.windows).length * 40) + 'px';
        win.style.top = (100 + Object.keys(this.windows).length * 40) + 'px';
        win.style.width = '800px';
        win.style.height = '600px';

        const titlebar = document.createElement('div');
        titlebar.className = 'window-titlebar';
        titlebar.addEventListener('mousedown', (event) => {
            CVKG.startDrag(event, name);
        });

        const title = document.createElement('div');
        title.className = 'window-title';
        title.textContent = name.toUpperCase();
        titlebar.appendChild(title);

        const controls = document.createElement('div');
        controls.className = 'window-controls';

        const minBtn = document.createElement('div');
        minBtn.className = 'window-btn min-btn';
        minBtn.addEventListener('click', () => CVKG.minimizeWindow(name));
        controls.appendChild(minBtn);

        const maxBtn = document.createElement('div');
        maxBtn.className = 'window-btn max-btn';
        maxBtn.addEventListener('click', () => CVKG.maximizeWindow(name));
        controls.appendChild(maxBtn);

        const closeBtn = document.createElement('div');
        closeBtn.className = 'window-btn close-btn';
        closeBtn.addEventListener('click', () => CVKG.closeWindow(name));
        controls.appendChild(closeBtn);

        titlebar.appendChild(controls);
        win.appendChild(titlebar);

        const content = document.createElement('div');
        content.className = 'window-content';
        content.id = `${name}-content`;
        win.appendChild(content);

        document.getElementById('desktop').appendChild(win);
        this.windows[name] = { element: win, minimized: false };
        this.focusWindow(name);
        this.loadAppContent(name);
    },

    loadAppContent(name, path = this.currentPath) {
        const content = document.getElementById(`${name}-content`);
        content.innerHTML = ''; // Clear previous content securely

        if (name === 'filemanager') {
            this.currentPath = path;
            const node = this.fs[path];

            const header = document.createElement('div');
            header.style.padding = '15px';
            header.style.borderBottom = '1px solid rgba(255,255,255,0.05)';
            header.style.display = 'flex';
            header.style.alignItems = 'center';
            header.style.gap = '10px';

            const backBtn = document.createElement('button');
            backBtn.textContent = '←';
            backBtn.style.background = 'none';
            backBtn.style.border = '1px solid var(--cyan)';
            backBtn.style.color = 'var(--cyan)';
            backBtn.style.padding = '2px 8px';
            backBtn.style.borderRadius = '4px';
            backBtn.style.cursor = 'pointer';
            backBtn.addEventListener('click', () => CVKG.navigateBack());

            const pathSpan = document.createElement('span');
            pathSpan.style.fontSize = '12px';
            pathSpan.style.opacity = '0.6';
            pathSpan.textContent = path;

            header.appendChild(backBtn);
            header.appendChild(pathSpan);
            content.appendChild(header);

            const grid = document.createElement('div');
            grid.style.padding = '20px';
            grid.style.display = 'grid';
            grid.style.gridTemplateColumns = 'repeat(auto-fill, minmax(110px, 1fr))';
            grid.style.gap = '20px';

            if (node && node.type === 'dir') {
                node.children.forEach(child => {
                    const childPath = path === '/' ? `/${child}` : `${path}/${child}`;
                    const isDir = this.fs[childPath].type === 'dir';

                    const item = document.createElement('div');
                    item.className = 'fm-item';
                    item.style.textAlign = 'center';
                    item.style.cursor = 'pointer';
                    item.style.padding = '10px';
                    item.style.borderRadius = '8px';

                    item.addEventListener('dblclick', () => CVKG.openFile(childPath));
                    item.addEventListener('contextmenu', (event) => {
                        event.preventDefault();
                        CVKG.showFileMenu(event, childPath);
                    });

                    const itemIcon = document.createElement('div');
                    itemIcon.style.fontSize = '32px';
                    itemIcon.style.marginBottom = '8px';
                    itemIcon.textContent = isDir ? '📂' : '📄';

                    const itemLabel = document.createElement('div');
                    itemLabel.style.fontSize = '11px';
                    itemLabel.style.color = '#fff';
                    itemLabel.textContent = child;

                    item.appendChild(itemIcon);
                    item.appendChild(itemLabel);
                    grid.appendChild(item);
                });
            }
            content.appendChild(grid);
        } else if (name === 'settings') {
            const container = document.createElement('div');
            container.style.padding = '30px';

            const title = document.createElement('h3');
            title.textContent = 'Desktop Settings';
            container.appendChild(title);

            const label = document.createElement('p');
            label.style.margin = '20px 0 10px';
            label.style.fontSize = '12px';
            label.style.opacity = '0.7';
            label.textContent = 'Background Image URL:';
            container.appendChild(label);

            const input = document.createElement('input');
            input.id = 'bg-url-input';
            input.type = 'text';
            input.placeholder = 'https://...';
            input.style.width = '100%';
            input.style.background = 'rgba(0,0,0,0.3)';
            input.style.border = '1px solid var(--cyan)';
            input.style.color = '#fff';
            input.style.padding = '8px';
            input.style.borderRadius = '4px';
            container.appendChild(input);

            const btn = document.createElement('button');
            btn.textContent = 'Apply Wallpaper';
            btn.style.marginTop = '20px';
            btn.style.background = 'var(--cyan)';
            btn.style.color = '#000';
            btn.style.border = 'none';
            btn.style.padding = '8px 20px';
            btn.style.borderRadius = '4px';
            btn.style.fontWeight = 'bold';
            btn.style.cursor = 'pointer';
            btn.addEventListener('click', () => CVKG.setWallpaper());
            container.appendChild(btn);

            content.appendChild(container);
        } else if (name === 'editor') {
            const fileContent = this.fs[this.editingFile]?.content || '';

            const container = document.createElement('div');
            container.style.height = '100%';
            container.style.display = 'flex';
            container.style.flexDirection = 'column';

            const header = document.createElement('div');
            header.style.padding = '10px';
            header.style.background = 'rgba(0,0,0,0.2)';
            header.style.fontSize = '12px';
            header.style.opacity = '0.6';
            header.style.borderBottom = '1px solid rgba(255,255,255,0.05)';
            header.textContent = this.editingFile || 'Untitled';
            container.appendChild(header);

            const textarea = document.createElement('textarea');
            textarea.id = 'editor-textarea';
            textarea.style.flex = '1';
            textarea.style.background = 'transparent';
            textarea.style.border = 'none';
            textarea.style.color = '#eee';
            textarea.style.padding = '20px';
            textarea.style.fontFamily = 'monospace';
            textarea.style.resize = 'none';
            textarea.style.outline = 'none';
            textarea.value = fileContent;
            container.appendChild(textarea);

            const footer = document.createElement('div');
            footer.style.padding = '10px';
            footer.style.borderTop = '1px solid rgba(255,255,255,0.05)';
            footer.style.textAlign = 'right';

            const saveBtn = document.createElement('button');
            saveBtn.textContent = 'Save File';
            saveBtn.style.background = 'var(--cyan)';
            saveBtn.style.color = '#000';
            saveBtn.style.border = 'none';
            saveBtn.style.padding = '4px 15px';
            saveBtn.style.borderRadius = '4px';
            saveBtn.style.fontSize = '12px';
            saveBtn.style.fontWeight = 'bold';
            saveBtn.style.cursor = 'pointer';
            saveBtn.addEventListener('click', () => CVKG.saveFile());
            footer.appendChild(saveBtn);
            container.appendChild(footer);

            content.appendChild(container);
        } else if (name === 'browser') {
            const container = document.createElement('div');
            container.style.height = '100%';
            container.style.display = 'flex';
            container.style.flexDirection = 'column';
            container.style.background = '#000';

            const nav = document.createElement('div');
            nav.style.padding = '10px';
            nav.style.background = 'rgba(20,20,40,0.8)';
            nav.style.display = 'flex';
            nav.style.gap = '10px';
            nav.style.borderBottom = '1px solid var(--cyan)';

            const input = document.createElement('input');
            input.id = 'browser-addr';
            input.type = 'text';
            input.value = 'https://www.duckduckgo.com';
            input.style.flex = '1';
            input.style.background = 'rgba(0,0,0,0.5)';
            input.style.border = '1px solid var(--cyan)';
            input.style.color = '#0ff';
            input.style.padding = '4px 12px';
            input.style.borderRadius = '20px';
            input.style.fontSize = '12px';
            input.style.outline = 'none';
            input.style.boxShadow = '0 0 5px rgba(0,255,255,0.2)';
            nav.appendChild(input);

            const goBtn = document.createElement('button');
            goBtn.textContent = 'Go';
            goBtn.style.background = 'var(--cyan)';
            goBtn.style.color = '#000';
            goBtn.style.border = 'none';
            goBtn.style.padding = '4px 20px';
            goBtn.style.borderRadius = '20px';
            goBtn.style.fontSize = '11px';
            goBtn.style.fontWeight = 'bold';
            goBtn.style.cursor = 'pointer';
            goBtn.style.textTransform = 'uppercase';
            
            const frameContainer = document.createElement('div');
            frameContainer.style.flex = '1';
            frameContainer.style.position = 'relative';
            frameContainer.style.background = '#111';

            const frame = document.createElement('iframe');
            frame.id = 'browser-frame';
            frame.src = 'https://www.duckduckgo.com';
            frame.style.width = '100%';
            frame.style.height = '100%';
            frame.style.border = 'none';
            frame.style.background = '#fff';

            goBtn.addEventListener('click', () => {
                // Harden M3 check: only allow self or verified https origins in frames
                const targetUrl = input.value;
                if (targetUrl.startsWith('https://') || targetUrl.startsWith('http://localhost') || targetUrl.startsWith('/')) {
                    frame.src = targetUrl;
                } else {
                    alert('Invalid Protocol: Only HTTPS links allowed.');
                }
            });

            nav.appendChild(goBtn);
            container.appendChild(nav);

            frameContainer.appendChild(frame);

            const note = document.createElement('div');
            note.id = 'browser-blocker-note';
            note.style.position = 'absolute';
            note.style.bottom = '10px';
            note.style.right = '10px';
            note.style.fontSize = '10px';
            note.style.color = 'var(--cyan)';
            note.style.opacity = '0.5';
            note.style.pointerEvents = 'none';
            note.textContent = 'Note: Some sites block embedding.';
            frameContainer.appendChild(note);

            container.appendChild(frameContainer);
            content.appendChild(container);
        } else if (name === 'imageviewer') {
            const container = document.createElement('div');
            container.style.height = '100%';
            container.style.display = 'flex';
            container.style.alignItems = 'center';
            container.style.justifyContent = 'center';
            container.style.background = '#000';

            const img = document.createElement('img');
            img.id = 'viewer-img';
            img.src = this.viewingImage;
            img.style.maxWidth = '100%';
            img.style.maxHeight = '100%';
            img.style.objectFit = 'contain';

            container.appendChild(img);
            content.appendChild(container);
        } else if (name === 'vikinggta') {
            this.initVikingGTA(content);
        } else if (name === 'terminal') {
            const output = document.createElement('div');
            output.id = 'term-output';
            output.style.height = 'calc(100% - 40px)';
            output.style.overflowY = 'auto';
            output.style.padding = '15px';
            output.style.fontFamily = 'monospace';
            output.style.fontSize = '12px';
            output.style.color = 'var(--cyan)';
            output.style.whiteSpace = 'pre-wrap';
            output.innerHTML = `Welcome to fish, the friendly interactive shell\nType 'help' for commands.\n\n<span style="color:#f0f">${escapeHTML(this.currentPath)}</span> > `;
            container = output; // temporary reference

            const input = document.createElement('input');
            input.id = 'term-input';
            input.type = 'text';
            input.style.width = '100%';
            input.style.height = '40px';
            input.style.background = 'rgba(0,0,0,0.3)';
            input.style.border = 'none';
            input.style.borderTop = '1px solid rgba(255,255,255,0.05)';
            input.style.color = '#fff';
            input.style.padding = '0 15px';
            input.style.fontFamily = 'monospace';
            input.style.outline = 'none';

            input.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    const cmd = input.value.trim();
                    this.executeTermCmd(cmd, output);
                    input.value = '';
                    output.scrollTop = output.scrollHeight;
                }
            });

            content.appendChild(output);
            content.appendChild(input);
            input.focus();
        } else {
            const fallback = document.createElement('div');
            fallback.style.padding = '20px';
            fallback.style.fontFamily = 'monospace';
            fallback.style.color = '#0f0';
            fallback.textContent = 'cvkg@os:~$ _';
            content.appendChild(fallback);
        }
    },

    initVikingGTA(container) {
        if (typeof THREE === 'undefined') {
            const err = document.createElement('div');
            err.style.padding = '20px';
            err.style.color = 'red';
            err.textContent = '[ ERROR ]: THREE.js failed to load. Check Content Security Policy.';
            container.appendChild(err);
            return;
        }
        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x050510);

        const camera = new THREE.PerspectiveCamera(75, 800 / 560, 0.1, 1000);
        camera.position.set(0, 10, 20);

        const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
        renderer.setSize(800, 560);
        container.appendChild(renderer.domElement);

        const worldSize = 200;
        const groundGeo = new THREE.PlaneGeometry(worldSize, worldSize);
        const groundMat = new THREE.MeshPhongMaterial({ color: 0x1a2a1a });
        const ground = new THREE.Mesh(groundGeo, groundMat);
        ground.rotation.x = -Math.PI / 2;
        scene.add(ground);

        const riverGeo = new THREE.PlaneGeometry(20, worldSize);
        const riverMat = new THREE.MeshPhongMaterial({
            color: 0x00aaff,
            transparent: true,
            opacity: 0.7,
            emissive: 0x004488
        });
        const river = new THREE.Mesh(riverGeo, riverMat);
        river.rotation.x = -Math.PI / 2;
        river.position.set(40, 0.1, 0);
        scene.add(river);

        const createHouse = (x, z) => {
            const houseGroup = new THREE.Group();
            const bodyGeo = new THREE.BoxGeometry(4, 4, 4);
            const bodyMat = new THREE.MeshPhongMaterial({ color: 0x442211 });
            const body = new THREE.Mesh(bodyGeo, bodyMat);
            body.position.y = 2;
            houseGroup.add(body);
            
            // Windows
            const winGeo = new THREE.PlaneGeometry(1, 1);
            const winMat = new THREE.MeshPhongMaterial({ color: 0x00ffff, emissive: 0x00ffff, emissiveIntensity: 0.5 });
            const w1 = new THREE.Mesh(winGeo, winMat); w1.position.set(0, 2, 2.01); houseGroup.add(w1);
            
            const roofGeo = new THREE.ConeGeometry(4, 3, 4);
            const roofMat = new THREE.MeshPhongMaterial({ color: 0x221100 });
            const roof = new THREE.Mesh(roofGeo, roofMat);
            roof.position.y = 5.5;
            roof.rotation.y = Math.PI / 4;
            houseGroup.add(roof);
            houseGroup.position.set(x, 0, z);
            scene.add(houseGroup);
        };

        for (let i = 0; i < 15; i++) {
            createHouse(-30 - Math.random() * 20, -100 + i * 15);
            createHouse(30 + Math.random() * 20, -100 + i * 15);
        }

        const boatGroup = new THREE.Group();
        const hullGeo = new THREE.BoxGeometry(8, 1.5, 4);
        const hullMat = new THREE.MeshPhongMaterial({ color: 0x331100 });
        const hull = new THREE.Mesh(hullGeo, hullMat);
        boatGroup.add(hull);
        
        // Dragon Head
        const headGeo = new THREE.BoxGeometry(1, 2, 1);
        const head = new THREE.Mesh(headGeo, hullMat);
        head.position.set(4.5, 1.5, 0);
        boatGroup.add(head);

        const mastGeo = new THREE.CylinderGeometry(0.15, 0.15, 7);
        const mastMat = new THREE.MeshPhongMaterial({ color: 0x553311 });
        const mast = new THREE.Mesh(mastGeo, mastMat);
        mast.position.y = 3.5;
        boatGroup.add(mast);
        const sailGeo = new THREE.PlaneGeometry(4, 5);
        const sailMat = new THREE.MeshPhongMaterial({ color: 0x992222, side: THREE.DoubleSide });
        const sail = new THREE.Mesh(sailGeo, sailMat);
        sail.position.set(0, 4.5, 0.1);
        boatGroup.add(sail);
        boatGroup.position.set(40, 0.5, 0);
        scene.add(boatGroup);

        const createVikingModel = (color) => {
            const group = new THREE.Group();
            const mat = new THREE.MeshPhongMaterial({ color: color, emissive: color, emissiveIntensity: 0.3 });
            
            // Head
            const head = new THREE.Mesh(new THREE.BoxGeometry(0.6, 0.6, 0.6), mat);
            head.position.y = 2.2;
            group.add(head);

            // Torso
            const torso = new THREE.Mesh(new THREE.BoxGeometry(1, 1.2, 0.5), mat);
            torso.position.y = 1.3;
            group.add(torso);

            // Arms
            const armGeo = new THREE.BoxGeometry(0.3, 1, 0.3);
            const la = new THREE.Mesh(armGeo, mat); la.position.set(-0.7, 1.3, 0); group.add(la);
            const ra = new THREE.Mesh(armGeo, mat); ra.position.set(0.7, 1.3, 0); group.add(ra);

            // Legs
            const legGeo = new THREE.BoxGeometry(0.4, 1, 0.4);
            const ll = new THREE.Mesh(legGeo, mat); ll.position.set(-0.3, 0.5, 0); group.add(ll);
            const rl = new THREE.Mesh(legGeo, mat); rl.position.set(0.3, 0.5, 0); group.add(rl);

            return group;
        };

        const player = createVikingModel(0xffff00);
        
        // Detailed Viking Shield
        const shield = new THREE.Mesh(new THREE.CylinderGeometry(0.7, 0.7, 0.1, 16), new THREE.MeshPhongMaterial({ color: 0x442211 }));
        shield.rotation.z = Math.PI / 2;
        shield.position.set(0.8, 1.3, 0.3);
        player.add(shield);

        // Detailed Viking Axe
        const handle = new THREE.Mesh(new THREE.CylinderGeometry(0.05, 0.05, 1.8), new THREE.MeshPhongMaterial({ color: 0x331100 }));
        handle.position.set(-0.8, 1.3, 0.3);
        handle.rotation.x = Math.PI / 4;
        player.add(handle);
        const blade = new THREE.Mesh(new THREE.BoxGeometry(0.1, 0.6, 0.7), new THREE.MeshPhongMaterial({ color: 0xcccccc }));
        blade.position.set(-0.8, 2.0, 0.6);
        player.add(blade);

        scene.add(player);

        // Spawning Other Vikings (Magenta) and Jarl (Blue)
        let jarlNPC = null;
        const spawnNPC = (x, z, color, isJarl = false) => {
            const npc = createVikingModel(color);
            npc.position.set(x, 0, z);
            scene.add(npc);
            if (isJarl) jarlNPC = npc;
        };

        for (let i = 0; i < 8; i++) {
            spawnNPC(-30 + Math.random() * 60, -120 + i * 20, 0xff00ff); // Magenta Vikings
        }
        spawnNPC(10, -50, 0x0000ff, true); // Blue Jarl

        const wantedUI = document.createElement('div');
        wantedUI.id = 'wanted-warning';
        wantedUI.textContent = 'WANTED: COMMITTING A CRIME';
        wantedUI.style.display = 'none';
        container.appendChild(wantedUI);

        const deathUI = document.createElement('div');
        deathUI.style.position = 'absolute';
        deathUI.style.top = '0';
        deathUI.style.left = '0';
        deathUI.style.width = '100%';
        deathUI.style.height = '100%';
        deathUI.style.background = 'rgba(0,0,0,0.8)';
        deathUI.style.display = 'none';
        deathUI.style.flexDirection = 'column';
        deathUI.style.alignItems = 'center';
        deathUI.style.justifyContent = 'center';
        deathUI.style.zIndex = '200';
        deathUI.style.color = '#fff';
        deathUI.style.fontFamily = 'Inter';

        const deathMsg = document.createElement('h1');
        deathMsg.id = 'death-msg';
        deathMsg.style.fontSize = '48px';
        deathMsg.style.color = '#ff3333';
        deathMsg.style.marginBottom = '20px';
        deathUI.appendChild(deathMsg);

        const respawnBtn = document.createElement('button');
        respawnBtn.id = 'respawn-btn';
        respawnBtn.style.background = 'var(--cyan)';
        respawnBtn.style.color = '#000';
        respawnBtn.style.border = 'none';
        respawnBtn.style.padding = '15px 40px';
        respawnBtn.style.borderRadius = '4px';
        respawnBtn.style.fontWeight = 'bold';
        respawnBtn.style.cursor = 'pointer';
        respawnBtn.textContent = 'CONTINUE (RESPAWN)';
        
        respawnBtn.addEventListener('click', () => {
            state.health = 2;
            state.wanted = false;
            state.gameOver = false;
            wantedUI.style.display = 'none';
            deathUI.style.display = 'none';
            player.position.set(0, 0, 0);
            boatGroup.position.set(40, 0.5, 0);
            state.mode = 'onFoot';
            player.visible = true;
            jarlNPC.position.set(10, 0, -50);
        });

        deathUI.appendChild(respawnBtn);
        container.appendChild(deathUI);

        const ambientLight = new THREE.AmbientLight(0x404040);
        scene.add(ambientLight);
        const pointLight = new THREE.PointLight(0xffff00, 1.5, 20);
        player.add(pointLight);
        pointLight.position.set(0, 2, 0);

        let state = { mode: 'onFoot', prompt: null, vy: 0, jumping: false, camRot: 0, camDist: 20, health: 2, wanted: false, gameOver: false };
        const gravity = -0.015;
        const keys = {};
        
        container.addEventListener('mousedown', (e) => {
            if (e.button === 2) state.isOrbiting = true;
        });
        window.addEventListener('mouseup', () => state.isOrbiting = false);
        container.addEventListener('mousemove', (e) => {
            if (state.isOrbiting) {
                state.camRot -= e.movementX * 0.01;
            }
        });
        container.oncontextmenu = (e) => e.preventDefault();

        const onKeyDown = (e) => {
            keys[e.code] = true;
            if (e.code === 'Space' && !state.jumping && state.mode === 'onFoot') {
                state.vy = 0.4;
                state.jumping = true;
            }
            if (e.code === 'KeyE') {
                const dist = player.position.distanceTo(boatGroup.position);
                if (dist < 5) {
                    state.mode = state.mode === 'onFoot' ? 'inBoat' : 'onFoot';
                    player.visible = state.mode === 'onFoot';
                    if (state.mode === 'inBoat') {
                        wantedUI.style.display = 'block';
                        state.wanted = true;
                    }
                    if (state.mode === 'onFoot') {
                        player.position.copy(boatGroup.position);
                        player.position.x -= 5;
                    }
                }
            }
        };
        const onKeyUp = (e) => keys[e.code] = false;
        window.addEventListener('keydown', onKeyDown);
        window.addEventListener('keyup', onKeyUp);

        const animate = () => {
            if (!container.parentNode) {
                window.removeEventListener('keydown', onKeyDown);
                window.removeEventListener('keyup', onKeyUp);
                return;
            }
            requestAnimationFrame(animate);
            const target = state.mode === 'onFoot' ? player : boatGroup;

            if (state.gameOver) return;

            // Jarl AI logic
            if (state.wanted && jarlNPC) {
                const jarlDir = target.position.clone().sub(jarlNPC.position).normalize();
                jarlNPC.position.add(jarlDir.multiplyScalar(0.15));
                jarlNPC.rotation.y = Math.atan2(jarlDir.x, jarlDir.z);
                
                const distToJarl = target.position.distanceTo(jarlNPC.position);
                if (distToJarl < 2) {
                    if (state.mode === 'inBoat') {
                        state.gameOver = true;
                        document.getElementById('death-msg').textContent = 'BLOOD EAGLED';
                        deathUI.style.display = 'flex';
                    } else {
                        state.health -= 0.05; // Slow drain during attack
                        if (state.health <= 0) {
                            state.gameOver = true;
                            document.getElementById('death-msg').textContent = 'SEE YOU IN VALHALLA';
                            deathUI.style.display = 'flex';
                        }
                    }
                }
            }

            if (state.mode === 'onFoot') {
                const forward = new THREE.Vector3();
                camera.getWorldDirection(forward);
                forward.y = 0;
                forward.normalize();
                const right = new THREE.Vector3().crossVectors(new THREE.Vector3(0, 1, 0), forward).normalize();

                const moveDir = new THREE.Vector3(0, 0, 0);
                if (keys['KeyW']) moveDir.add(forward);
                if (keys['KeyS']) moveDir.sub(forward);
                if (keys['KeyA']) moveDir.add(right);
                if (keys['KeyD']) moveDir.sub(right);

                if (moveDir.length() > 0) {
                    moveDir.normalize().multiplyScalar(0.2);
                    const nextPos = player.position.clone().add(moveDir);
                    
                    const distToBoat = nextPos.distanceTo(boatGroup.position);
                    if (distToBoat > 4.5) {
                        player.position.copy(nextPos);
                        player.rotation.y = Math.atan2(moveDir.x, moveDir.z);
                    }
                }
                state.vy += gravity;
                player.position.y += state.vy;
                if (player.position.y <= 0) {
                    player.position.y = 0;
                    state.vy = 0;
                    state.jumping = false;
                }
            } else {
                 const speed = 0.4;
                 if (keys['KeyW']) boatGroup.position.z -= speed;
                 if (keys['KeyS']) boatGroup.position.z += speed;
                 if (keys['KeyA']) boatGroup.position.x -= speed;
                 if (keys['KeyD']) boatGroup.position.x += speed;
            }

            const distToBoat = player.position.distanceTo(boatGroup.position);
            if (distToBoat < 5 && state.mode === 'onFoot') {
                if (!state.prompt) {
                    state.prompt = document.createElement('div');
                    state.prompt.style.position = 'absolute';
                    state.prompt.style.top = '20px';
                    state.prompt.style.left = '50%';
                    state.prompt.style.transform = 'translateX(-50%)';
                    state.prompt.style.background = 'rgba(0,0,0,0.8)';
                    state.prompt.style.color = 'var(--cyan)';
                    state.prompt.style.padding = '10px 20px';
                    state.prompt.style.borderRadius = '20px';
                    state.prompt.style.border = '1px solid var(--cyan)';
                    state.prompt.textContent = 'PRESS [E] TO STEAL BOAT';
                    container.appendChild(state.prompt);
                }
            } else if (state.prompt) {
                state.prompt.remove();
                state.prompt = null;
            }
            camera.position.x = target.position.x + Math.sin(state.camRot) * state.camDist;
            camera.position.z = target.position.z + Math.cos(state.camRot) * state.camDist;
            camera.position.y = target.position.y + 10;
            camera.lookAt(target.position);
            renderer.render(scene, camera);
        };
        animate();
    },

    focusWindow(name) {
        const win = this.windows[name];
        if (!win) return;
        win.element.style.zIndex = ++this.zIndex;
        win.minimized = false;
        win.element.style.display = 'flex';
    },

    minimizeWindow(name) {
        this.windows[name].element.style.display = 'none';
        this.windows[name].minimized = true;
    },

    maximizeWindow(name) {
        const win = this.windows[name].element;
        if (win.style.width !== '100%') {
            win.dataset.oldStyle = win.getAttribute('style');
            win.style.left = '0';
            win.style.top = '0';
            win.style.width = '100%';
            win.style.height = '100%';
            win.style.borderRadius = '0';
        } else {
            win.setAttribute('style', win.dataset.oldStyle);
        }
    },

    closeWindow(name) {
        this.windows[name].element.remove();
        delete this.windows[name];
    },

    startDrag(e, name) {
        const win = this.windows[name].element;
        this.focusWindow(name);
        const rect = win.getBoundingClientRect();
        const shiftX = e.clientX - rect.left;
        const shiftY = e.clientY - rect.top;
        function moveAt(pageX, pageY) {
            win.style.left = pageX - shiftX + 'px';
            win.style.top = pageY - shiftY + 'px';
        }
        function onMouseMove(event) { moveAt(event.clientX, event.clientY); }
        document.addEventListener('mousemove', onMouseMove);
        document.onmouseup = function () {
            document.removeEventListener('mousemove', onMouseMove);
            document.onmouseup = null;
        };
    },

    async updateClock() {
        try {
            const response = await fetch('/api/system/time');
            const data = await response.json();
            const now = new Date(data.timestamp * 1000);
            document.getElementById('system-tray-time').textContent = now.toLocaleTimeString();
        } catch (e) {
            const now = new Date();
            document.getElementById('system-tray-time').textContent = now.toLocaleTimeString();
        }
    },

    setupEventListeners() {
        document.addEventListener('click', (e) => {
            document.getElementById('context-menu').style.display = 'none';
            document.getElementById('file-menu').style.display = 'none';
            if (!e.target.closest('#start-menu') && !e.target.closest('.dock-icon')) {
                document.getElementById('start-menu').classList.remove('active');
            }
        });
    },

    showContextMenu(e) {
        if (e.target.closest('.fm-item')) return; // File menu handles this
        e.preventDefault();
        const menu = document.getElementById('context-menu');
        menu.style.display = 'block';
        menu.style.left = (e.clientX - 100) + 'px';
        menu.style.top = (e.clientY - 100) + 'px';
    },

    showFileMenu(e, path) {
        e.preventDefault();
        e.stopPropagation();
        this.selectedFilePath = path;
        const menu = document.getElementById('file-menu');
        menu.style.display = 'block';
        menu.style.left = e.clientX + 'px';
        menu.style.top = e.clientY + 'px';
    },

    fileAction(action) {
        if (!this.selectedFilePath) return;
        const parts = this.selectedFilePath.split('/');
        const name = parts.pop();
        const parentPath = parts.join('/') || '/';

        if (action === 'Delete') {
            this.fs[parentPath].children = this.fs[parentPath].children.filter(c => c !== name);
            delete this.fs[this.selectedFilePath];
            this.loadAppContent('filemanager', parentPath);
        } else if (action === 'Rename') {
            const newName = prompt('Enter new name:', name);
            if (newName && newName !== name) {
                this.fs[parentPath].children = this.fs[parentPath].children.map(c => c === name ? newName : c);
                const newNodePath = parentPath === '/' ? `/${newName}` : `${parentPath}/${newName}`;
                this.fs[newNodePath] = this.fs[this.selectedFilePath];
                delete this.fs[this.selectedFilePath];
                this.loadAppContent('filemanager', parentPath);
            }
        } else if (action === 'Copy') {
            this.clipboard = this.selectedFilePath;
        } else if (action === 'Move') {
            if (this.clipboard) {
                const cParts = this.clipboard.split('/');
                const cName = cParts.pop();
                const cParent = cParts.join('/') || '/';
                this.fs[cParent].children = this.fs[cParent].children.filter(c => c !== cName);
                this.fs[parentPath].children.push(cName);
                const newPath = parentPath === '/' ? `/${cName}` : `${parentPath}/${cName}`;
                this.fs[newPath] = this.fs[this.clipboard];
                delete this.fs[this.clipboard];
                this.clipboard = null;
                this.loadAppContent('filemanager', parentPath);
            }
        } else if (action === 'Properties') {
            const node = this.fs[this.selectedFilePath];
            alert(`Path: ${this.selectedFilePath}\nType: ${node.type}\nContent: ${node.type === 'file' ? node.content.length + ' bytes' : node.children.length + ' items'}`);
        }
        document.getElementById('file-menu').style.display = 'none';
    },

    menuAction(action) {
        document.getElementById('context-menu').style.display = 'none';
        if (action === 'Settings') {
            this.openApp('settings');
        } else if (action === 'New File') {
            const name = 'New_File_' + Date.now() + '.txt';
            this.fs['/Documents'].children.push(name);
            this.fs['/Documents/' + name] = { type: 'file', content: '' };
            this.openApp('filemanager');
            this.loadAppContent('filemanager', '/Documents');
        } else if (action === 'New Folder') {
            const name = 'New_Folder_' + Date.now();
            this.fs['/'].children.push(name);
            this.fs['/' + name] = { type: 'dir', children: [] };
            this.openApp('filemanager');
            this.loadAppContent('filemanager', '/');
        } else if (action === 'Refresh') {
            window.location.reload();
        }
    },

    navigateBack() {
        if (this.currentPath === '/') return;
        const parts = this.currentPath.split('/');
        parts.pop();
        const parent = parts.join('/') || '/';
        this.loadAppContent('filemanager', parent);
    },

    setWallpaper() {
        const url = document.getElementById('bg-url-input').value;
        if (url) {
            document.getElementById('desktop').style.background = `url('${url}') center/cover no-repeat`;
        }
    },

    openFile(path) {
        const node = this.fs[path];
        if (node.type === 'dir') {
            this.loadAppContent('filemanager', path);
        } else if (path.endsWith('.txt') || path.endsWith('.rs')) {
            this.editingFile = path;
            this.openApp('editor');
        } else if (path.endsWith('.png') || path.endsWith('.jpg')) {
            this.viewingImage = path;
            this.openApp('imageviewer');
        }
    },

    saveFile() {
        const content = document.getElementById('editor-textarea').value;
        if (this.editingFile) {
            this.fs[this.editingFile].content = content;
        }
    },

    executeTermCmd(input, output) {
        const parts = input.split(' ');
        const cmd = parts[0].toLowerCase();
        const arg = parts[1];
        const arg2 = parts[2];
        let out = `\n<span style="color:#f0f">${escapeHTML(this.currentPath)}</span> > ${escapeHTML(input)}\n`;

        const getFullPath = (p) => {
            if (!p) return this.currentPath;
            if (p.startsWith('/')) return p;
            return this.currentPath === '/' ? `/${p}` : `${this.currentPath}/${p}`;
        };

        if (cmd === 'ls') {
            const node = this.fs[this.currentPath];
            out += node.children.map(c => {
                const isDir = this.fs[getFullPath(c)].type === 'dir';
                return isDir ? `<span style="color:#0ff">[DIR]  ${escapeHTML(c)}</span>` : `<span style="color:#eee">[FILE] ${escapeHTML(c)}</span>`;
            }).join('\n');
        } else if (cmd === 'cd') {
            if (!arg || arg === '/') {
                this.currentPath = '/';
            } else if (arg === '..') {
                const p = this.currentPath.split('/');
                p.pop();
                this.currentPath = p.join('/') || '/';
            } else {
                const target = getFullPath(arg);
                if (this.fs[target] && this.fs[target].type === 'dir') {
                    this.currentPath = target;
                } else {
                    out += `cd: no such directory: ${escapeHTML(arg)}`;
                }
            }
        } else if (cmd === 'pwd') {
            out += escapeHTML(this.currentPath);
        } else if (cmd === 'echo') {
            out += escapeHTML(parts.slice(1).join(' '));
        } else if (cmd === 'touch') {
            const target = getFullPath(arg);
            if (!this.fs[target]) {
                this.fs[this.currentPath].children.push(arg);
                this.fs[target] = { type: 'file', content: '' };
            }
        } else if (cmd === 'rm') {
            const target = getFullPath(arg);
            if (this.fs[target]) {
                const pParts = target.split('/');
                const name = pParts.pop();
                const parent = pParts.join('/') || '/';
                this.fs[parent].children = this.fs[parent].children.filter(c => c !== name);
                delete this.fs[target];
            } else {
                out += `rm: no such file or directory: ${escapeHTML(arg)}`;
            }
        } else if (cmd === 'mv') {
            const src = getFullPath(arg);
            const dest = getFullPath(arg2);
            if (this.fs[src]) {
                this.fs[dest] = this.fs[src];
                delete this.fs[src];
                out += `Moved ${escapeHTML(arg)} to ${escapeHTML(arg2)}`;
            }
        } else if (cmd === 'cat') {
            const target = getFullPath(arg);
            if (this.fs[target] && this.fs[target].type === 'file') {
                out += escapeHTML(this.fs[target].content);
            } else {
                out += `cat: no such file: ${escapeHTML(arg)}`;
            }
        } else if (cmd === 'mkdir') {
            const target = getFullPath(arg);
            if (!this.fs[target]) {
                this.fs[this.currentPath].children.push(arg);
                this.fs[target] = { type: 'dir', children: [] };
            } else {
                out += `mkdir: directory already exists: ${escapeHTML(arg)}`;
            }
        } else if (cmd === 'help') {
            out += 'fish commands: ls, cd, pwd, echo, touch, rm, mv, cat, mkdir, clear, help';
        } else if (cmd === 'clear') {
            output.innerHTML = `Welcome to fish, the friendly interactive shell\nType 'help' for commands.\n\n<span style="color:#f0f">${escapeHTML(this.currentPath)}</span> > `;
            return;
        } else if (cmd) {
            out += `Unknown command: ${escapeHTML(cmd)}`;
        }
        output.innerHTML += out + `\n\n<span style="color:#f0f">${escapeHTML(this.currentPath)}</span> > `;
    }
};

CVKG.init();
window.CVKG = CVKG;
