using UnityEngine;

namespace IAIDO.Audio
{
    public sealed class AudioController : MonoBehaviour
    {
        [Header("Clips")] public AudioSource ambient;
        public AudioSource goCue;
        public AudioSource swordDraw;
        public AudioSource hit;
        public AudioSource clash;

        public void PlayAmbient(bool on)
        {
            if (ambient == null) return;
            if (on && !ambient.isPlaying) ambient.Play();
            if (!on && ambient.isPlaying) ambient.Stop();
        }

        public void PlayGoCue() { if (goCue != null) goCue.Play(); }
        public void OnHit() { if (hit != null) hit.Play(); }
        public void OnClash() { if (clash != null) clash.Play(); }
        public void OnDraw() { if (swordDraw != null) swordDraw.Play(); }
    }
}

