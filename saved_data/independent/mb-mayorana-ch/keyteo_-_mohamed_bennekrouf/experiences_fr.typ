#import "template.typ": conf, date, dated_experience, experience_details, section, get_text


#let get_key_insights() = {
  (
    "Experienced technical professional with proven track record",
    "Expert in modern development technologies and methodologies", 
    "Strong background in project delivery and team collaboration",
    "Passionate about innovative solutions and continuous learning"
  )
}

#let get_work_experience() = {
  [
    == Company Name
    #dated_experience(
      "Job Title",
      date: "Start Date - End Date",
      description: "Brief company description",
      content: [
        #experience_details(
          "Key responsibility or achievement with specific metrics if possible"
        )
        #experience_details(
          "Another responsibility focusing on technical leadership or delivery"
        )
        #experience_details(
          "Additional responsibility highlighting impact or problem-solving"
        )
      ]
    )
    
    == Previous Company
    #dated_experience(
      "Previous Job Title",
      date: "Start Date - End Date", 
      description: "Brief description of previous company",
      content: [
        #experience_details(
          "Previous role key responsibility"
        )
        #experience_details(
          "Another responsibility from previous experience"
        )
      ]
    )
  ]
}
