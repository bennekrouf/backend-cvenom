#import "../templates/default/template.typ": *

// Expériences en français
#let get_work_experience() = {
  // Ajoutez vos expériences professionnelles ici
  [
    #experience(
      title: "Votre poste",
      date: "2023 - Présent",
      description: "Nom de l'entreprise",
      details: [
        - Vos responsabilités et réalisations
        - Ajoutez plus de points selon vos besoins
      ]
    )
  ]
}

#let get_key_insights() = {
  [
    - Point clé ou réalisation #1
    - Point clé ou réalisation #2
    - Point clé ou réalisation #3
  ]
}

#let get_skills() = {
  (
    "Langages": ("Rust", "Python", "JavaScript"),
    "Frameworks": ("React", "Vue.js", "FastAPI"),
    "Outils": ("Git", "Docker", "CI/CD"),
  )
}

#let get_education() = {
  [
    #experience(
      title: "Votre diplôme",
      date: "2019 - 2023",
      description: "Université/École",
      details: [
        - Spécialisation ou mention
        - Projets remarquables
      ]
    )
  ]
}

#let get_languages() = {
  (
    "Français": "Langue maternelle",
    "Anglais": "Courant",
    "Allemand": "Notions",
  )
}
