import{$ as e,F as t,Ft as n,G as r,Ht as i,It as a,L as o,M as s,V as c,Vt as l,ct as u,g as d,gt as f,ht as ee,l as te,mt as p,r as m,z as h}from"../chunks/cBdug0a_.js";import"../chunks/xihTtKlq.js";import{b as g,i as _}from"../chunks/BK8tric1.js";import{d as v,i as y,t as b}from"../chunks/CrTMul68.js";import{n as x,t as S}from"../chunks/B-1wecjy.js";import{t as C}from"../chunks/BpWmU0gq.js";import{t as w}from"../chunks/MiYR6i5Y.js";import{t as T}from"../chunks/BoHlb8qN.js";import{t as E}from"../chunks/CXeOpnMt.js";import{t as D}from"../chunks/CsPWb3fg.js";function O(e){return e&&typeof e==`object`&&!Array.isArray(e)?e:null}function k(e){if(typeof e==`string`&&e.trim())return e.trim();let t=O(e);return t&&typeof t.value==`string`&&t.value.trim()?t.value.trim():null}function A(e){let t=O(e)??{},n=O(t.data)??t,r=O(n.booking)??{},i=O(r.responses)??O(n.responses)??{},a=O((Array.isArray(r.attendees)?r.attendees:[])[0])??{},o=O(n.eventType)??{};return{email:k(a.email)??k(i.email)??k(n.attendeeEmail)??k(n.email),name:k(a.name)??k(i.name)??k(n.attendeeName)??k(n.name),eventType:k(o.slug)??k(o.title)??k(n.eventTypeSlug),startsAt:k(r.startTime)??k(n.date)??k(n.startTime)}}function j(e,t){return{email:e.email??t.email,name:e.name??t.name,eventType:e.eventType??t.eventType,startsAt:e.startsAt??t.startsAt}}var M=1500;function N(e,t=M){let n=null,r=null,i=new Set,a=()=>{if(r=null,!n)return;let t=n;n=null,t.startsAt&&i.add(t.startsAt),e(t)};return{receive(e){let o=A(e);!n&&o.startsAt&&i.has(o.startsAt)||(n=n?j(n,o):o,r??=setTimeout(a,t))},flush(){r&&clearTimeout(r),a()}}}function P(e){_(`Demo Booked`,{attendee_email:e.email,attendee_name:e.name,event_type:e.eventType,booking_starts_at:e.startsAt})}var F=c(`<li class="flex items-start gap-3"><span class="mt-[7px] size-2 shrink-0 bg-fern-600" aria-hidden="true"></span> <span> </span></li>`),I=r(c(`<div class="h-full w-full overflow-scroll" id="my-cal-inline-ferndesk"></div> <script type="text/javascript">
							(function (C, A, L) {
								let p = function (a, ar) {
									a.q.push(ar);
								};
								let d = C.document;
								C.Cal =
									C.Cal ||
									function () {
										let cal = C.Cal;
										let ar = arguments;
										if (!cal.loaded) {
											cal.ns = {};
											cal.q = cal.q || [];
											d.head.appendChild(d.createElement('script')).src = A;
											cal.loaded = true;
										}
										if (ar[0] === L) {
											const api = function () {
												p(api, arguments);
											};
											const namespace = ar[1];
											api.q = api.q || [];
											if (typeof namespace === 'string') {
												cal.ns[namespace] = cal.ns[namespace] || api;
												p(cal.ns[namespace], ar);
												p(cal, ['initNamespace', namespace]);
											} else p(cal, ar);
											return;
										}
										p(cal, ar);
									};
							})(window, 'https://app.cal.com/embed/embed.js', 'init');
							Cal('init', 'ferndesk', { origin: 'https://app.cal.com' });

							Cal.ns.ferndesk('inline', {
								elementOrSelector: '#my-cal-inline-ferndesk',
								config: { layout: 'month_view', useSlotsViewOnSmallScreen: 'true', theme: 'light' },
								calLink: 'wilson/ferndesk-discovery'
							});

							Cal.ns.ferndesk('ui', {
								theme: 'light',
								cssVarsPerTheme: { light: { 'cal-brand': '#1C1B19' } },
								hideEventTypeDetails: true,
								layout: 'month_view'
							});

							// Hand confirmed bookings to the page (see onMount) for analytics. Cal
							// sends both events for one booking; the page records it once.
							['bookingSuccessful', 'bookingSuccessfulV2'].forEach(function (action) {
								Cal.ns.ferndesk('on', {
									action: action,
									callback: function (event) {
										window.dispatchEvent(
											new CustomEvent('ferndesk:demo-booked', { detail: event.detail })
										);
									}
								});
							});
						<\/script>`,1)),L=c(`<!> <!> <!> <main class="bg-bg"><section class="relative overflow-hidden"><!> <div class="relative container-marketing pt-16 pb-28 lg:pt-24 lg:pb-36"><div class="grid gap-10 lg:grid-cols-[1fr_minmax(0,460px)] lg:items-start lg:gap-20"><div class="max-w-[520px] lg:pt-1"><span class="eyebrow block">Book a demo</span> <h1 class="mt-4 font-display text-display-sm text-balance lg:text-display-md">See how easy it is to get set up.</h1> <p class="mt-4 text-lede text-pretty text-prose">Pick a time below. Wilson takes you through the whole setup, from importing your current
						help center to connecting Fern to the tools you already use. A one-click import is
						usually live in about ten minutes.</p> <ul class="mt-7 flex flex-col gap-3 border-t border-border pt-7 text-md text-prose"></ul> <div class="mt-7 flex items-center gap-3"><img alt="Wilson, who runs Ferndesk demos" width="40" height="40" loading="lazy" class="size-10 shrink-0 rounded object-cover"/> <span class="flex flex-col leading-tight"><span class="text-md font-semibold text-fg">Wilson</span> <span class="text-base text-fg-3">Founder, Ferndesk</span></span></div></div> <div class="overflow-hidden rounded-md border border-border-strong bg-bg-raised shadow-float"><!></div></div></div></section> <!> <!></main> <!> <!>`,1);function R(r,c){a(c,!1);let _=[`Your help center imported, internal links fixed, redirects set up`,`Fern connected to your GitHub, Linear and your support tool`,`Screenshots taken from your own product, and re-taken when it changes`,`Hosting on your domain or /help, pricing, and any question you have`],O=`Book a demo`,k=`See how Ferndesk keeps your help center up to date with AI. Schedule a personalized demo.`,A=[v({name:O,description:k,path:`/book-a-demo`}),y([{name:`Home`,url:`/`},{name:`Book a demo`,url:`/book-a-demo`}])];m(()=>{let e=N(P),t=t=>e.receive(t.detail),n=()=>e.flush();return window.addEventListener(`ferndesk:demo-booked`,t),window.addEventListener(`pagehide`,n),()=>{window.removeEventListener(`ferndesk:demo-booked`,t),window.removeEventListener(`pagehide`,n),e.flush()}}),te();var j=L(),M=ee(j);b(M,{get schema(){return A}});var R=f(M,2);T(R,{title:O,description:k});var z=f(R,2);S(z,{});var B=f(z,2),V=p(B),H=p(V);E(H,{class:`max-md:hidden`,height:`4rem`,fadeTo:`var(--bg)`});var U=f(H,2),W=p(U),G=p(W),K=f(p(G),6);s(K,5,()=>_,e=>e,(t,n)=>{var r=F(),a=f(p(r),2),s=p(a,!0);i(a),i(r),u(()=>o(s,e(n))),h(t,r)}),i(K);var q=f(K,2),J=p(q);l(2),i(q),i(G);var Y=f(G,2),X=p(Y),Z=e=>{var t=I();l(2),h(e,t)};t(X,e=>{e(Z)}),i(Y),i(W),i(U),i(V);var Q=f(V,2);w(Q,{}),D(f(Q,2),{}),i(B);var $=f(B,2);C($,{module:`demo_page_conversion`}),x(f($,2),{}),u(()=>d(J,`src`,g)),h(r,j),n()}export{R as component};