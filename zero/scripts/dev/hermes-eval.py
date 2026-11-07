import sys,os,subprocess as S,hashlib,time
def stop(m):
    print("STOP: "+m);sys.exit(1)
d=globals().get("d");f=None
if d is None:
    f=os.path.abspath(__file__);d=open(f,encoding="utf-8").read();a=sys.argv[1:]
else:
    a=sys.argv[2:]
H=hashlib.sha256(d.encode()).hexdigest()[:12]
c=os.path.expanduser("~/.cache/zero")
if f is None:
    os.makedirs(c,exist_ok=True)
    open(c+"/hermes-eval-"+H+".py","w",encoding="utf-8").write(d)
    print("saved this launcher: "+c+"/hermes-eval-"+H+".py")
    print("run it with: python3 "+c+"/hermes-eval-"+H+".py "+chr(34)+"your task"+chr(34))
    sys.exit(0)
if os.path.basename(f)!="hermes-eval-"+H+".py":stop("file name does not match its hash.")
if len(a)!=1 or not a[0].strip():stop("give exactly one argument, the task text in double quotes.")
h=os.path.expanduser("~");ev=h+"/.cache/zero/hermes-eval";hh=h+"/.hermes";src=h+"/0-core";rp=ev+"/work/repo"
if not os.path.isdir(rp+"/.git") or not os.path.isdir(ev+"/home"):stop("the evaluation home or the repo copy is missing.")
def g(p,*x):
    return S.run(["git","-C",p]+list(x),capture_output=True,text=True).stdout
pr=S.run(["curl","-s","-m","5","-o","/dev/null","-w","%{http_code}","-H","Authorization: Bearer dummy-eval-token","http://127.0.0.1:8645/v1/models"],capture_output=True,text=True).stdout.strip()
if pr!="200":stop("the proxy is not answering (http "+pr+"). In another terminal run: hermes proxy start --provider nous")
before=(g(src,"rev-parse","HEAD").strip(),g(src,"status","--short"))
bw=["bwrap","--ro-bind","/","/","--tmpfs",h,"--tmpfs","/tmp","--tmpfs","/run","--ro-bind-try","/run/systemd/resolve","/run/systemd/resolve","--dev","/dev","--proc","/proc","--ro-bind",hh+"/hermes-agent",hh+"/hermes-agent","--ro-bind",hh+"/tools",hh+"/tools","--ro-bind",h+"/.local/bin/hermes",h+"/.local/bin/hermes","--bind",ev+"/home",ev+"/home","--bind",ev+"/work",ev+"/work","--chdir",rp,"--unshare-pid","--unshare-ipc","--new-session","--die-with-parent","--clearenv","--setenv","PATH",h+"/.local/bin:/usr/bin:/bin","--setenv","HOME",h,"--setenv","HERMES_HOME",ev+"/home","--setenv","TERM","xterm","--setenv","LANG","C.UTF-8","hermes","-t","terminal,file,todo,clarify","-z",a[0]]
t0=time.time()
try:
    r=S.run(bw,capture_output=True,text=True,timeout=900);rc=r.returncode;out=r.stdout;err=r.stderr
except S.TimeoutExpired:
    rc=-1;out="";err="TIMEOUT after 900 seconds"
after=(g(src,"rev-parse","HEAD").strip(),g(src,"status","--short"))
R=["task: "+a[0],"toolsets: terminal,file,todo,clarify (no code_execution, web, browser, memory, cronjob, delegation)","seconds: %d  rc: %d" % (time.time()-t0,rc),"--- HERMES ANSWER ---",out.rstrip(),"--- HERMES STDERR, last 600 chars ---",err[-600:],"--- CHANGES IN THE COPY ---",g(rp,"status","--short").rstrip() or "none","diff stat: "+(g(rp,"diff","--stat").strip() or "none"),"--- YOUR REAL REPO ---","~/0-core HEAD unchanged: %s" % (before[0]==after[0]),"~/0-core status unchanged: %s" % (before[1]==after[1])]
os.makedirs(ev+"/runs",exist_ok=True)
lf=ev+"/runs/"+time.strftime("%Y%m%d-%H%M%S")+".txt"
open(lf,"w",encoding="utf-8").write(chr(10).join(R)+chr(10))
print(chr(10).join(R))
print("log: "+lf)